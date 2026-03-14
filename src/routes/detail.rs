use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

const DETAIL_TTL: u64 = 60 * 60 * 24; // 24 hours

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/novel/{type}/{id}/detail", get(get_detail))
}

#[utoipa::path(
    get,
    path = "/api/novel/{type}/{id}/detail",
    tag = "小説情報",
    summary = "小説詳細取得",
    description = "小説のタイトル・あらすじ・総ページ数を取得する。結果は24時間キャッシュされる。外部サイトへの取得は最大3回リトライ（500ms × 試行回数のバックオフ）。",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
    ),
    responses(
        (status = 200, description = "小説の詳細情報", body = crate::openapi::DetailResponse,
            example = json!({"title": "小説タイトル", "synopsis": "あらすじテキスト...", "page": 150})),
        (status = 400, description = "無効なサイト種別", body = crate::openapi::ErrorResponse),
        (status = 502, description = "外部サイトからの取得に失敗（3回リトライ後）", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_detail(
    State(state): State<AppState>,
    Path((type_str, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let key = format!("novel:{}:{}:detail", type_str, id);

    if let Some(cached) = state.cache.get(&key) {
        return Ok(Json(cached));
    }

    let label = format!("fetchDetail {}/{}", type_str, id);
    let detail = super::with_retry(&label, || module.fetch_detail(&state.http, &id)).await?;
    state.cache.set(&key, detail.clone(), Some(DETAIL_TTL));
    Ok(Json(detail))
}
