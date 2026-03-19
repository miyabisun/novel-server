use crate::auth::UserId;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/rss", get(get_rss))
}

struct FeedItem {
    type_str: String,
    id: String,
    title: String,
    novelupdated_at: Option<String>,
    page: i64,
    read: i64,
}

#[utoipa::path(
    get,
    path = "/api/rss",
    tag = "RSS",
    summary = "お気に入り更新RSSフィード",
    description = "お気に入り小説のうち、未読が1〜9話（0 < 総ページ数 - 既読ページ < 10）の小説の更新情報をRSS 2.0形式で配信する。更新日時の降順。読み切った小説は表示されない。",
    responses(
        (status = 200, description = "RSS 2.0 XML", content_type = "application/rss+xml"),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_rss(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let items = {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT type, id, title, novelupdated_at, page, read FROM favorites
             WHERE user_id = ?1 AND page - read > 0 AND page - read < 10
             ORDER BY novelupdated_at DESC NULLS LAST",
        )?;
        let rows = stmt.query_map([user_id.0], |row| {
            Ok(FeedItem {
                type_str: row.get(0)?,
                id: row.get(1)?,
                title: row.get(2)?,
                novelupdated_at: row.get(3)?,
                page: row.get(4)?,
                read: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let base = resolve_base_url(&headers, &state.config);

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
<title>Novel Server - お気に入り更新</title>
<description>お気に入り小説の更新情報</description>
"#,
    );
    xml.push_str(&format!("<link>{}</link>\n", escape_xml(&base)));

    for item in &items {
        xml.push_str(&build_item_xml(item, &base));
    }

    xml.push_str("</channel>\n</rss>");

    Ok(([(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")], xml))
}

/// Derive the base URL from request headers (reverse proxy or direct access).
fn resolve_base_url(headers: &HeaderMap, config: &crate::config::Config) -> String {
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    let default_host = format!("localhost:{}", config.port);
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|v| v.to_str().ok())
        .unwrap_or(&default_host);

    format!("{}://{}{}", proto, host, config.base_path)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn build_item_xml(item: &FeedItem, base: &str) -> String {
    let next_page = (item.read + 1).min(item.page.max(1));
    let link = format!("{}/novel/{}/{}/{}", base, item.type_str, item.id, next_page);
    let mut xml = String::from("<item>\n");
    xml.push_str(&format!("<title>{}</title>\n", escape_xml(&item.title)));
    xml.push_str(&format!("<link>{}</link>\n", escape_xml(&link)));
    xml.push_str(&format!(
        "<description>{}話 / 既読{}話</description>\n",
        item.page, item.read
    ));
    if let Some(ref dt) = item.novelupdated_at {
        xml.push_str(&format!("<pubDate>{}</pubDate>\n", escape_xml(dt)));
    }
    xml.push_str(&format!(
        "<guid>{}/{}/{}</guid>\n",
        escape_xml(base),
        escape_xml(&item.type_str),
        escape_xml(&item.id)
    ));
    xml.push_str("</item>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_xml_special_chars() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("<script>"), "&lt;script&gt;");
        assert_eq!(escape_xml(r#"he said "hi""#), "he said &quot;hi&quot;");
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn escape_xml_no_special_chars() {
        assert_eq!(escape_xml("hello world"), "hello world");
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn build_item_xml_basic() {
        let item = FeedItem {
            type_str: "narou".into(),
            id: "n1234ab".into(),
            title: "Test Novel".into(),
            novelupdated_at: Some("2026-03-14T00:00:00".into()),
            page: 100,
            read: 98,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("<title>Test Novel</title>"));
        assert!(xml.contains("<link>http://localhost:3000/novel/narou/n1234ab/99</link>"),
            "read=98, page=100 should link to 99");
        assert!(xml.contains("<description>100話 / 既読98話</description>"));
        assert!(xml.contains("<pubDate>2026-03-14T00:00:00</pubDate>"));
        assert!(xml.contains("<guid>http://localhost:3000/narou/n1234ab</guid>"));
    }

    #[test]
    fn build_item_xml_without_updated_at() {
        let item = FeedItem {
            type_str: "kakuyomu".into(),
            id: "abc".into(),
            title: "Novel".into(),
            novelupdated_at: None,
            page: 50,
            read: 48,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(!xml.contains("<pubDate>"));
    }

    fn test_config(base_path: &str) -> crate::config::Config {
        crate::config::Config {
            port: 3000,
            base_path: base_path.to_string(),
            db_path: String::new(),
        }
    }

    #[test]
    fn resolve_base_url_direct_access() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "localhost:3000".parse().unwrap());
        let config = test_config("");
        assert_eq!(resolve_base_url(&headers, &config), "http://localhost:3000");
    }

    #[test]
    fn resolve_base_url_reverse_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", "https".parse().unwrap());
        headers.insert("x-forwarded-host", "novels.example.com".parse().unwrap());
        let config = test_config("");
        assert_eq!(
            resolve_base_url(&headers, &config),
            "https://novels.example.com"
        );
    }

    #[test]
    fn resolve_base_url_with_base_path() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", "https".parse().unwrap());
        headers.insert("x-forwarded-host", "example.com".parse().unwrap());
        let config = test_config("/novels");
        assert_eq!(
            resolve_base_url(&headers, &config),
            "https://example.com/novels"
        );
    }

    #[test]
    fn resolve_base_url_no_headers() {
        let headers = HeaderMap::new();
        let config = test_config("");
        assert_eq!(resolve_base_url(&headers, &config), "http://localhost:3000");
    }

    #[test]
    fn build_item_xml_links_to_next_page() {
        let item = FeedItem {
            type_str: "narou".into(),
            id: "n1".into(),
            title: "Novel".into(),
            novelupdated_at: None,
            page: 10,
            read: 0,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("/n1/1</link>"), "read=0 should link to page 1");

        let item2 = FeedItem {
            type_str: "narou".into(),
            id: "n2".into(),
            title: "Novel 2".into(),
            novelupdated_at: None,
            page: 10,
            read: 5,
        };
        let xml2 = build_item_xml(&item2, "http://localhost:3000");
        assert!(xml2.contains("/n2/6</link>"), "read=5 should link to page 6");
    }

    #[test]
    fn build_item_xml_clamps_to_max_page() {
        let item = FeedItem {
            type_str: "narou".into(),
            id: "n1".into(),
            title: "Novel".into(),
            novelupdated_at: None,
            page: 100,
            read: 100,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("/n1/100</link>"),
            "read=100, page=100 should clamp to 100, not 101");
    }

    #[test]
    fn build_item_xml_escapes_title() {
        let item = FeedItem {
            type_str: "narou".into(),
            id: "n1".into(),
            title: "Title <with> & \"special\" chars".into(),
            novelupdated_at: None,
            page: 10,
            read: 9,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("Title &lt;with&gt; &amp; &quot;special&quot; chars"));
    }
}
