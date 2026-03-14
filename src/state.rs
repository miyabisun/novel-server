use crate::cache::Cache;
use crate::config::Config;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    /// SQLite は高速なので非同期プール不要。Mutex guard は {} ブロック内で
    /// 完結させ .await を跨がないこと（跨ぐと Send 制約違反でコンパイルエラー）。
    pub db: Arc<Mutex<Connection>>,
    pub cache: Arc<Cache>,
    pub config: Config,
    pub http: reqwest::Client,
}
