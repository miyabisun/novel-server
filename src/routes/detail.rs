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
