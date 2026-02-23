mod cache;
mod config;
mod db;
mod error;
mod modules;
mod routes;
mod sanitize;
mod spa;
mod state;
mod sync;

use config::Config;
use state::AppState;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let conn = db::open(&config.db_path);
    let cache = Arc::new(cache::Cache::new());
    let http = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to build HTTP client");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        cache: cache.clone(),
        config: config.clone(),
        http,
    };

    cache::start_sweep(cache);
    sync::start_sync(state.clone());

    let app = routes::build_router(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    tracing::info!(
        "Server running on http://localhost:{}{}",
        config.port,
        if config.base_path.is_empty() {
            "/".to_string()
        } else {
            config.base_path.clone()
        }
    );
    axum::serve(listener, app).await.expect("Server error");
}
