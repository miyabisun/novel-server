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

    for i in 0..3u32 {
        match module.fetch_detail(&state.http, &id).await {
            Ok(detail) => {
                state.cache.set(&key, detail.clone(), Some(DETAIL_TTL));
                return Ok(Json(detail));
            }
            Err(e) => {
                tracing::error!("fetchDetail {}/{} attempt {} failed: {}", type_str, id, i + 1, e);
                if i < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (i as u64 + 1)))
                        .await;
                }
            }
        }
    }
    Err(AppError::Upstream("Failed to fetch detail".into()))
}
