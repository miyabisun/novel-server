use crate::error::AppError;
use crate::modules::ModuleType;
use crate::sanitize;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde_json::{json, Value};

const PAGE_TTL: u64 = 60 * 60 * 24; // 24 hours

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/novel/{type}/{id}/pages/{num}", get(get_page))
        .route("/api/novel/{type}/{id}/pages/{num}", patch(patch_page))
}

#[utoipa::path(
    get,
    path = "/api/novel/{type}/{id}/pages/{num}",
    tag = "小説本文",
    summary = "ページ本文取得",
    description = "小説の本文HTMLを取得する。結果は24時間キャッシュされる。外部サイトへの取得は最大3回リトライ。\n\nHTMLは許可リスト方式でサニタイズ済み（p, br, div, span, ruby等のコンテンツタグのみ許可。許可外のタグと全属性は除去）。",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
        ("num" = String, Path, description = "ページ番号（1始まり）。kakuyomuの場合はエピソードIDも使用可", example = "1"),
    ),
    responses(
        (status = 200, description = "サニタイズ済みHTML本文", body = crate::openapi::PageResponse,
            example = json!({"html": "<p>本文のHTML...</p>"})),
        (status = 400, description = "無効なサイト種別", body = crate::openapi::ErrorResponse),
        (status = 502, description = "外部サイトからの取得に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_page(
    State(state): State<AppState>,
    Path((type_str, id, num)): Path<(String, String, String)>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let key = format!("novel:{}:{}:page:{}", type_str, id, num);

    if let Some(cached) = state.cache.get(&key) {
        return Ok(Json(json!({ "html": cached })));
    }

    fetch_and_cache(&state, &module, &id, &num, &key).await
}

#[utoipa::path(
    patch,
    path = "/api/novel/{type}/{id}/pages/{num}",
    tag = "小説本文",
    summary = "ページ本文再取得（キャッシュ無視）",
    description = "キャッシュを無視してページ本文を再取得する。レスポンス形式はGETと同一。",
    params(
        ("type" = String, Path, description = "対象サイト", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
        ("num" = String, Path, description = "ページ番号", example = "1"),
    ),
    responses(
        (status = 200, description = "サニタイズ済みHTML本文", body = crate::openapi::PageResponse),
        (status = 400, description = "無効なサイト種別", body = crate::openapi::ErrorResponse),
        (status = 502, description = "外部サイトからの取得に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn patch_page(
    State(state): State<AppState>,
    Path((type_str, id, num)): Path<(String, String, String)>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let key = format!("novel:{}:{}:page:{}", type_str, id, num);

    fetch_and_cache(&state, &module, &id, &num, &key).await
}

async fn fetch_and_cache(
    state: &AppState,
    module: &ModuleType,
    id: &str,
    num: &str,
    key: &str,
) -> Result<Json<Value>, AppError> {
    let label = format!("fetchPage {}/{}/{}", id, num, key);
    let raw = super::with_retry(&label, || module.fetch_page(&state.http, id, num)).await?;
    let html = sanitize::clean(raw.as_deref().unwrap_or(""));
    state
        .cache
        .set(key, Value::String(html.clone()), Some(PAGE_TTL));
    Ok(Json(json!({ "html": html })))
}
