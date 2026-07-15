use crate::config::Config;
use axum::http::{header, HeaderMap};
use rusqlite::Connection;

pub(super) struct FavoriteUpdate {
    pub type_str: String,
    pub id: String,
    pub title: String,
    pub novelupdated_at: Option<String>,
    pub page: i64,
    pub read: i64,
}

pub(super) fn load_favorite_updates(
    db: &Connection,
    user_id: i64,
) -> rusqlite::Result<Vec<FavoriteUpdate>> {
    let mut stmt = db.prepare(
        "SELECT type, id, title, novelupdated_at, page, read FROM favorites
         WHERE user_id = ?1 AND page - read > 0 AND page - read < 10
         ORDER BY novelupdated_at DESC NULLS LAST",
    )?;
    let items = stmt
        .query_map([user_id], |row| {
            Ok(FavoriteUpdate {
                type_str: row.get(0)?,
                id: row.get(1)?,
                title: row.get(2)?,
                novelupdated_at: row.get(3)?,
                page: row.get(4)?,
                read: row.get(5)?,
            })
        })?
        .collect();
    items
}

pub(super) fn resolve_base_url(headers: &HeaderMap, config: &Config) -> String {
    if let Some(url) = &config.public_base_url {
        return url.clone();
    }

    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");

    let default_host = format!("localhost:{}", config.port);
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())
        .unwrap_or(&default_host);

    format!("{}://{}{}", proto, host, config.base_path)
}

pub(super) fn next_page(read: i64, page: i64) -> i64 {
    (read + 1).min(page.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(base_path: &str) -> Config {
        Config {
            port: 3000,
            base_path: base_path.to_string(),
            public_base_url: None,
            db_path: String::new(),
        }
    }

    #[test]
    fn resolves_direct_request_url() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "localhost:3000".parse().unwrap());

        assert_eq!(
            resolve_base_url(&headers, &test_config("")),
            "http://localhost:3000"
        );
    }

    #[test]
    fn resolves_reverse_proxy_url_with_base_path() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", "https".parse().unwrap());
        headers.insert("x-forwarded-host", "example.com".parse().unwrap());

        assert_eq!(
            resolve_base_url(&headers, &test_config("/novels")),
            "https://example.com/novels"
        );
    }

    #[test]
    fn falls_back_when_request_has_no_host() {
        assert_eq!(
            resolve_base_url(&HeaderMap::new(), &test_config("")),
            "http://localhost:3000"
        );
    }

    #[test]
    fn configured_public_url_takes_precedence() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "novel:3000".parse().unwrap());
        let mut config = test_config("");
        config.public_base_url = Some("https://novel.sis.jp".to_string());

        assert_eq!(resolve_base_url(&headers, &config), "https://novel.sis.jp");
    }

    #[test]
    fn next_page_is_first_unread_and_clamped_to_existing_range() {
        assert_eq!(next_page(0, 10), 1);
        assert_eq!(next_page(5, 10), 6);
        assert_eq!(next_page(100, 100), 100);
        assert_eq!(next_page(0, 0), 1);
    }
}
