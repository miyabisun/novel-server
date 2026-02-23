use crate::cache::Cache;
use crate::config::Config;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub cache: Arc<Cache>,
    pub config: Config,
    pub http: reqwest::Client,
}
