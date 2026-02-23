use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

const RANKING_TTL: u64 = 60 * 60 * 3; // 3 hours
const VALID_PERIODS: &[&str] = &["daily", "weekly", "monthly", "quarter", "yearly"];
const QUARTER_UNSUPPORTED: &[&str] = &["kakuyomu"];

#[derive(Deserialize)]
struct RankingQuery {
    period: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/novel/{type}/ranking", get(get_ranking))
        .route("/api/novel/{type}/ranking", patch(patch_ranking))
}

async fn get_ranking(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<RankingQuery>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let period = query.period.as_deref().unwrap_or("daily");
    validate_period(&type_str, period)?;

    let key = format!("novel:{}:ranking:{}", type_str, period);

    if let Some(cached) = state.cache.get(&key) {
        return Ok(Json(cached));
    }

    let ranking = module
        .fetch_ranking_list(&state.http, 100, period)
        .await
        .map_err(|_| AppError::Upstream("Failed to fetch ranking".into()))?;
    state.cache.set(&key, ranking.clone(), Some(RANKING_TTL));
    Ok(Json(ranking))
}

async fn patch_ranking(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<RankingQuery>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let period = query.period.as_deref().unwrap_or("daily");
    validate_period(&type_str, period)?;

    let key = format!("novel:{}:ranking:{}", type_str, period);

    let ranking = module
        .fetch_ranking_list(&state.http, 100, period)
        .await
        .map_err(|_| AppError::Upstream("Failed to fetch ranking".into()))?;
    state.cache.set(&key, ranking.clone(), Some(RANKING_TTL));
    Ok(Json(ranking))
}

fn validate_period(type_str: &str, period: &str) -> Result<(), AppError> {
    if !VALID_PERIODS.contains(&period) {
        return Err(AppError::BadRequest("Invalid period".into()));
    }
    if period == "quarter" && QUARTER_UNSUPPORTED.contains(&type_str) {
        return Err(AppError::BadRequest(format!(
            "{} does not support quarter ranking",
            type_str
        )));
    }
    Ok(())
}
