mod detail;
mod favorites;
mod pages;
mod ranking;
mod search;
mod toc;

use crate::error::AppError;
use crate::spa;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::future::Future;
use tower_http::services::ServeDir;

async fn with_retry<F, Fut, T>(label: &str, f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    for i in 0..3u32 {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                tracing::error!("{} attempt {} failed: {}", label, i + 1, e);
                if i < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (i as u64 + 1)))
                        .await;
                }
            }
        }
    }
    Err(AppError::Upstream(format!("Failed after 3 retries: {}", label)))
}

pub fn build_router(state: AppState) -> Router {
    let base_path = state.config.base_path.clone();

    let api = Router::new()
        .merge(ranking::routes())
        .merge(pages::routes())
        .merge(detail::routes())
        .merge(favorites::routes())
        .merge(search::routes())
        .merge(toc::routes());

    let sub = Router::new()
        .merge(api)
        .nest_service("/assets", ServeDir::new("client/build/assets"))
        .nest_service(
            "/favicon.svg",
            tower_http::services::ServeFile::new("client/build/favicon.svg"),
        )
        .fallback(get(move || {
            let bp = base_path.clone();
            async move { spa_fallback(&bp) }
        }))
        .with_state(state.clone());

    let app_base = state.config.base_path.clone();
    if app_base.is_empty() {
        sub
    } else {
        Router::new().nest(&app_base, sub)
    }
}

fn spa_fallback(base_path: &str) -> impl IntoResponse {
    match spa::get_index_html(base_path) {
        Some(html) => Html(html).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "Frontend not built. Run: cd client && npm install && npx vite build"})),
        )
            .into_response(),
    }
}
