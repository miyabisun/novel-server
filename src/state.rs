use crate::cache::Cache;
use crate::config::Config;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    /// No async pool needed — SQLite is fast enough with a simple Mutex.
    /// Keep the guard within a {} block; holding it across .await violates Send.
    pub db: Arc<Mutex<Connection>>,
    pub cache: Arc<Cache>,
    pub config: Config,
    pub http: reqwest::Client,
}
