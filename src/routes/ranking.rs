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

#[utoipa::path(
    get,
    path = "/api/novel/{type}/ranking",
    tag = "ランキング",
    summary = "ランキング取得",
    description = "指定サイトのランキングをジャンル別にグループ化して取得する。「総合」キーにジャンル横断の総合ランキングを含む。結果は3時間キャッシュされる。\n\n## 対応サイト\n- **narou**: 小説家になろう（daily/weekly/monthly/quarter/yearly）\n- **nocturne**: ノクターンノベルズ（daily/weekly/monthly/quarter/yearly）\n- **kakuyomu**: カクヨム（daily/weekly/monthly/yearly）※ quarterは非対応",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("period" = Option<String>, Query, description = "期間（デフォルト: daily）", example = "daily"),
    ),
    responses(
        (status = 200, description = "「総合」キーとジャンル名をキーとし、各キーに小説の配列が入ったオブジェクト", body = Object,
            example = json!({"総合": [{"id": "n1234ab", "title": "小説タイトル", "page": 150, "noveltype": 1}], "ハイファンタジー": [{"id": "n5678cd", "title": "別の小説", "page": 50, "noveltype": 1}]})),
        (status = 400, description = "無効なパラメータ", body = crate::openapi::ErrorResponse,
            example = json!({"error": "Invalid period"})),
        (status = 502, description = "外部サイトからの取得に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_ranking(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<RankingQuery>,
) -> Result<Json<Value>, AppError> {
    fetch_ranking(state, &type_str, query.period.as_deref(), true).await
}

#[utoipa::path(
    patch,
    path = "/api/novel/{type}/ranking",
    tag = "ランキング",
    summary = "ランキング再取得（キャッシュ無視）",
    description = "キャッシュを無視してランキングを再取得する。パラメータ・レスポンス形式はGETと同一。",
    params(
        ("type" = String, Path, description = "対象サイト", example = "narou"),
        ("period" = Option<String>, Query, description = "期間（デフォルト: daily）", example = "weekly"),
    ),
    responses(
        (status = 200, description = "ランキングデータ", body = Object),
        (status = 400, description = "無効なパラメータ", body = crate::openapi::ErrorResponse),
        (status = 502, description = "外部サイトからの取得に失敗", body = crate::openapi::ErrorResponse),
    ),
)]
async fn patch_ranking(
    State(state): State<AppState>,
    Path(type_str): Path<String>,
    Query(query): Query<RankingQuery>,
) -> Result<Json<Value>, AppError> {
    fetch_ranking(state, &type_str, query.period.as_deref(), false).await
}

async fn fetch_ranking(
    state: AppState,
    type_str: &str,
    period: Option<&str>,
    use_cache: bool,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(type_str)?;
    let period = period.unwrap_or("daily");
    validate_period(type_str, period)?;

    let key = format!("novel:{}:ranking:{}", type_str, period);

    if use_cache {
        if let Some(cached) = state.cache.get(&key) {
            return Ok(Json(cached));
        }
    }

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
