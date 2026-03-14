use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/novel/{type}/{id}/toc", get(get_toc))
}

#[utoipa::path(
    get,
    path = "/api/novel/{type}/{id}/toc",
    tag = "小説情報",
    summary = "目次取得",
    description = "小説の目次（全エピソード一覧）を取得する。キャッシュなし（リアルタイム性を重視し、最新の話数を即時反映）。外部サイトへの取得は最大3回リトライ。\n\nnarou / nocturne はトップページ（目次ページ）をスクレイピング、kakuyomu は Apollo State から抽出する。",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
    ),
    responses(
        (status = 200, description = "目次情報", body = crate::openapi::TocResponse,
            example = json!({"title": "小説タイトル", "episodes": [{"num": 1, "title": "第1話 タイトル"}, {"num": 2, "title": "第2話 タイトル"}]})),
        (status = 400, description = "無効なサイト種別", body = crate::openapi::ErrorResponse),
        (status = 502, description = "外部サイトからの取得に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_toc(
    State(state): State<AppState>,
    Path((type_str, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let label = format!("fetchToc {}/{}", type_str, id);
    let toc = super::with_retry(&label, || module.fetch_toc(&state.http, &id)).await?;
    Ok(Json(toc))
}
