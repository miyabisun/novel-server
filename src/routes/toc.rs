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

async fn get_toc(
    State(state): State<AppState>,
    Path((type_str, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let label = format!("fetchToc {}/{}", type_str, id);
    let toc = super::with_retry(&label, || module.fetch_toc(&state.http, &id)).await?;
    Ok(Json(toc))
}
