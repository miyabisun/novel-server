use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub base_path: String,
    /// Canonical public origin for feed links. Includes BASE_PATH when needed.
    pub public_base_url: Option<String>,
    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let base_path = env::var("BASE_PATH")
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_string();

        if !base_path.is_empty() {
            let re = regex_lite::Regex::new(r"^/[\w\-/]*$").unwrap();
            if !re.is_match(&base_path) {
                panic!("Invalid BASE_PATH: {}", base_path);
            }
        }

        let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "/data/novel.db".to_string());

        let public_base_url = env::var("PUBLIC_BASE_URL")
            .ok()
            .map(|url| url.trim().trim_end_matches('/').to_string())
            .filter(|url| !url.is_empty());

        Self {
            port,
            base_path,
            public_base_url,
            db_path,
        }
    }
}
