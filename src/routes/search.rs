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

async fn get_search(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
    ModuleType::resolve(&type_str)?;
    let q = query
        .q
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("Missing query parameter: q".into()))?;

    let module = ModuleType::resolve(&type_str)?;
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
