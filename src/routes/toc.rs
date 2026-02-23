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

    for i in 0..3u32 {
        match module.fetch_toc(&state.http, &id).await {
            Ok(toc) => return Ok(Json(toc)),
            Err(e) => {
                tracing::error!("fetchToc {}/{} attempt {} failed: {}", type_str, id, i + 1, e);
                if i < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (i as u64 + 1)))
                        .await;
                }
            }
        }
    }
    Err(AppError::Upstream(
        "Failed to fetch table of contents".into(),
    ))
}
