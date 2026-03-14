use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

const SEARCH_TTL: u64 = 60 * 60; // 1 hour

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/novel/{type}/search", get(get_search))
}

#[utoipa::path(
    get,
    path = "/api/novel/{type}/search",
    tag = "検索",
    summary = "小説検索",
    description = "キーワードで小説を検索する。最大20件、評価順。結果は1時間キャッシュされる。",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("q" = String, Query, description = "検索キーワード（必須）", example = "異世界"),
    ),
    responses(
        (status = 200, description = "検索結果の配列", body = Vec<crate::openapi::SearchItem>,
            example = json!([{"id": "n1234ab", "title": "異世界転生物語", "page": 150}])),
        (status = 400, description = "検索パラメータ不正", body = crate::openapi::ErrorResponse,
            example = json!({"error": "Missing query parameter: q"})),
        (status = 502, description = "外部サイトからの検索に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_search(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let q = query
        .q
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing query parameter: q".into()))?;
    let key = format!("novel:{}:search:{}", type_str, q);

    if let Some(cached) = state.cache.get(&key) {
        return Ok(Json(cached));
    }

    let results = module
        .fetch_search(&state.http, q)
        .await
        .map_err(|_| AppError::Upstream("Failed to search".into()))?;
    state.cache.set(&key, results.clone(), Some(SEARCH_TTL));
    Ok(Json(results))
}
