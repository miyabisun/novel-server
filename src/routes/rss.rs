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

use super::feed::{load_favorite_updates, next_page, resolve_base_url, FavoriteUpdate};

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
    let items = load_favorite_updates(&state.db.lock().unwrap(), user_id.0)?;

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

    Ok((
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    ))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn build_item_xml(item: &FavoriteUpdate, base: &str) -> String {
    let next_page = next_page(item.read, item.page);
    let link = format!("{}/novel/{}/{}/{}", base, item.type_str, item.id, next_page);
    let mut xml = String::from("<item>\n");
    xml.push_str(&format!("<title>{}</title>\n", escape_xml(&item.title)));
    xml.push_str(&format!("<link>{}</link>\n", escape_xml(&link)));
    xml.push_str(&format!(
        "<description>{}話 / 既読{}話</description>\n",
        item.page, item.read
    ));
    if let Some(dt) = item
        .novelupdated_at
        .and_then(crate::time::unix_timestamp_to_rfc2822)
    {
        xml.push_str(&format!("<pubDate>{}</pubDate>\n", escape_xml(&dt)));
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
        let item = FavoriteUpdate {
            type_str: "narou".into(),
            id: "n1234ab".into(),
            title: "Test Novel".into(),
            novelupdated_at: Some(1_773_446_400),
            page: 100,
            read: 98,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("<title>Test Novel</title>"));
        assert!(
            xml.contains("<link>http://localhost:3000/novel/narou/n1234ab/99</link>"),
            "read=98, page=100 should link to 99"
        );
        assert!(xml.contains("<description>100話 / 既読98話</description>"));
        assert!(xml.contains("<pubDate>Sat, 14 Mar 2026 00:00:00 +0000</pubDate>"));
        assert!(xml.contains("<guid>http://localhost:3000/narou/n1234ab</guid>"));
    }

    #[test]
    fn build_item_xml_without_updated_at() {
        let item = FavoriteUpdate {
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

    #[test]
    fn build_item_xml_links_to_next_page() {
        let item = FavoriteUpdate {
            type_str: "narou".into(),
            id: "n1".into(),
            title: "Novel".into(),
            novelupdated_at: None,
            page: 10,
            read: 0,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(xml.contains("/n1/1</link>"), "read=0 should link to page 1");

        let item2 = FavoriteUpdate {
            type_str: "narou".into(),
            id: "n2".into(),
            title: "Novel 2".into(),
            novelupdated_at: None,
            page: 10,
            read: 5,
        };
        let xml2 = build_item_xml(&item2, "http://localhost:3000");
        assert!(
            xml2.contains("/n2/6</link>"),
            "read=5 should link to page 6"
        );
    }

    #[test]
    fn build_item_xml_clamps_to_max_page() {
        let item = FavoriteUpdate {
            type_str: "narou".into(),
            id: "n1".into(),
            title: "Novel".into(),
            novelupdated_at: None,
            page: 100,
            read: 100,
        };
        let xml = build_item_xml(&item, "http://localhost:3000");
        assert!(
            xml.contains("/n1/100</link>"),
            "read=100, page=100 should clamp to 100, not 101"
        );
    }

    #[test]
    fn build_item_xml_escapes_title() {
        let item = FavoriteUpdate {
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
