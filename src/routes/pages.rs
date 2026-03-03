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
