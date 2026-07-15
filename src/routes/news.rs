use super::feed::{load_favorite_updates, next_page, resolve_base_url, FavoriteUpdate};
use crate::auth::UserId;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::{header, HeaderMap};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/news", get(get_news))
}

#[utoipa::path(
    get,
    path = "/api/news",
    tag = "RSS",
    summary = "お気に入り更新の JSON Feed 1.1",
    description = "お気に入り小説のうち、未読が1〜9話（0 < 総ページ数 - 既読ページ < 10）の小説の更新情報を JSON Feed 1.1 形式で配信する。/api/rss と同じ抽出条件。news-server が定期取得して統合タイムラインに載せる。item の url は「次に読む話」へ直接リンクし、拡張フィールド `_news` に話数・既読数を含む。",
    responses(
        (status = 200, description = "JSON Feed 1.1", content_type = "application/feed+json"),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_news(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let items = load_favorite_updates(&state.db.lock().unwrap(), user_id.0)?;

    let base = resolve_base_url(&headers, &state.config);
    let feed = build_json_feed(&items, &base);

    Ok((
        [(header::CONTENT_TYPE, "application/feed+json; charset=utf-8")],
        axum::Json(feed),
    ))
}

fn build_json_feed(items: &[FavoriteUpdate], base: &str) -> serde_json::Value {
    json!({
        "version": "https://jsonfeed.org/version/1.1",
        "title": "Novel Server - お気に入り更新",
        "home_page_url": base,
        "items": items.iter().map(|item| build_item(item, base)).collect::<Vec<_>>(),
    })
}

fn build_item(item: &FavoriteUpdate, base: &str) -> serde_json::Value {
    let unread = item.page - item.read;
    let mut obj = json!({
        "id": format!("{}/{}", item.type_str, item.id),
        "url": format!(
            "{}/novel/{}/{}/{}",
            base,
            item.type_str,
            item.id,
            next_page(item.read, item.page)
        ),
        "title": item.title,
        "content_text": format!("{}話 / 既読{}話", item.page, item.read),
        "_news": {
            "service": "novel",
            "type": item.type_str,
            "total": item.page,
            "read": item.read,
            "unread": unread,
        },
    });
    if let Some(dt) = item.novelupdated_at.as_deref().and_then(to_rfc3339_jst) {
        obj["date_published"] = json!(dt);
    }
    obj
}

/// Normalize `novelupdated_at` to RFC3339 as JSON Feed requires.
/// Source sites (なろう/カクヨム) store naive JST timestamps like
/// `2026-03-14T00:00:00`; tag them with +09:00. Values that already carry a
/// timezone are passed through, anything unparseable is dropped (the item is
/// then published without a date rather than with an invalid one).
fn to_rfc3339_jst(s: &str) -> Option<String> {
    use chrono::{DateTime, FixedOffset, NaiveDateTime};

    let s = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.to_rfc3339());
    }
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .ok()?;
    let jst = FixedOffset::east_opt(9 * 3600).expect("+09:00 is a valid offset");
    Some(naive.and_local_timezone(jst).single()?.to_rfc3339())
}

#[cfg(test)]
mod tests {
    // News Feed Spec (JSON Feed 1.1)
    //
    // GET /api/news delivers favorite-novel updates as JSON Feed 1.1 for the
    // news-server aggregator. Same selection as /api/rss:
    // - Only favorites with 1-9 unread pages (0 < page - read < 10)
    // - Sorted by novelupdated_at DESC NULLS LAST
    // - item.url points at the next unread page
    // - date_published is RFC3339: naive JST timestamps get +09:00, absent or
    //   malformed timestamps omit the field entirely
    //
    // Handler tests drive the real `get_news` over HTTP (oneshot) against an
    // in-memory DB so the SQL and user scoping are what is under test.

    use super::*;
    use crate::cache::Cache;
    use crate::config::Config;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        AppState {
            db: Arc::new(Mutex::new(crate::db::open(":memory:"))),
            cache: Arc::new(Cache::new()),
            config: Config {
                port: 3000,
                base_path: String::new(),
                public_base_url: None,
                db_path: String::new(),
            },
            http: reqwest::Client::new(),
        }
    }

    fn insert_favorite(
        state: &AppState,
        user_id: i64,
        type_str: &str,
        id: &str,
        updated_at: Option<&str>,
        page: i64,
        read: i64,
    ) {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT OR IGNORE INTO users (id, email) VALUES (?1, 'u' || ?1)",
            [user_id],
        )
        .unwrap();
        db.execute(
            "INSERT INTO favorites (user_id, type, id, title, novelupdated_at, page, read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                user_id,
                type_str,
                id,
                format!("Novel {}", id),
                updated_at,
                page,
                read
            ],
        )
        .unwrap();
    }

    async fn get_news_feed(state: &AppState, user_id: i64) -> (StatusCode, serde_json::Value) {
        let app = Router::new()
            .route("/api/news", get(get_news))
            .layer(Extension(UserId(user_id)))
            .with_state(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/news")
                    .header("x-forwarded-proto", "https")
                    .header("x-forwarded-host", "novel.example.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert!(
            content_type.starts_with("application/feed+json"),
            "content-type must be application/feed+json, got: {content_type}"
        );
        let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    fn item_ids(feed: &serde_json::Value) -> Vec<String> {
        feed["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["id"].as_str().unwrap().to_string())
            .collect()
    }

    #[tokio::test]
    async fn news_includes_only_favorites_with_1_to_9_unread() {
        let state = test_state();
        // 0 unread → excluded; 1-9 unread → included; 10+ unread → excluded
        insert_favorite(
            &state,
            1,
            "narou",
            "n_done",
            Some("2026-03-14T00:00:00"),
            10,
            10,
        );
        insert_favorite(
            &state,
            1,
            "narou",
            "n_read",
            Some("2026-03-13T00:00:00"),
            10,
            5,
        );
        insert_favorite(
            &state,
            1,
            "narou",
            "n_far",
            Some("2026-03-12T00:00:00"),
            30,
            10,
        );

        let (status, feed) = get_news_feed(&state, 1).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(item_ids(&feed), vec!["narou/n_read"]);
    }

    #[tokio::test]
    async fn news_sorted_by_updated_desc_nulls_last() {
        let state = test_state();
        insert_favorite(
            &state,
            1,
            "narou",
            "n_old",
            Some("2026-03-01T00:00:00"),
            10,
            9,
        );
        insert_favorite(
            &state,
            1,
            "narou",
            "n_new",
            Some("2026-03-14T00:00:00"),
            10,
            9,
        );
        insert_favorite(&state, 1, "kakuyomu", "k_null", None, 10, 9);

        let (_, feed) = get_news_feed(&state, 1).await;
        assert_eq!(
            item_ids(&feed),
            vec!["narou/n_new", "narou/n_old", "kakuyomu/k_null"]
        );
    }

    #[tokio::test]
    async fn news_scopes_feed_to_the_authenticated_user() {
        let state = test_state();
        insert_favorite(
            &state,
            1,
            "narou",
            "n_u1",
            Some("2026-03-14T00:00:00"),
            10,
            9,
        );
        insert_favorite(
            &state,
            2,
            "narou",
            "n_u2",
            Some("2026-03-14T00:00:00"),
            10,
            9,
        );

        let (_, feed1) = get_news_feed(&state, 1).await;
        assert_eq!(item_ids(&feed1), vec!["narou/n_u1"]);

        let (_, feed2) = get_news_feed(&state, 2).await;
        assert_eq!(item_ids(&feed2), vec!["narou/n_u2"]);
    }

    #[tokio::test]
    async fn news_items_follow_json_feed_1_1_with_news_extension() {
        let state = test_state();
        insert_favorite(
            &state,
            1,
            "narou",
            "n1234ab",
            Some("2026-03-14T00:00:00"),
            100,
            98,
        );

        let (_, feed) = get_news_feed(&state, 1).await;
        assert_eq!(feed["version"], "https://jsonfeed.org/version/1.1");
        assert_eq!(feed["home_page_url"], "https://novel.example.com");

        let item = &feed["items"][0];
        assert_eq!(item["id"], "narou/n1234ab");
        assert_eq!(
            item["url"], "https://novel.example.com/novel/narou/n1234ab/99",
            "read=98, page=100 must link to next unread page 99"
        );
        assert_eq!(item["title"], "Novel n1234ab");
        assert_eq!(
            item["content_text"], "100話 / 既読98話",
            "JSON Feed 1.1 requires content_text or content_html on every item"
        );
        assert_eq!(
            item["date_published"], "2026-03-14T00:00:00+09:00",
            "naive JST timestamp must be tagged with +09:00"
        );
        assert_eq!(item["_news"]["service"], "novel");
        assert_eq!(item["_news"]["type"], "narou");
        assert_eq!(item["_news"]["total"], 100);
        assert_eq!(item["_news"]["read"], 98);
        assert_eq!(item["_news"]["unread"], 2);
    }

    #[tokio::test]
    async fn news_uses_configured_public_url_for_links() {
        let mut state = test_state();
        state.config.public_base_url = Some("https://novel.sis.jp".to_string());
        insert_favorite(&state, 1, "narou", "n1234ab", None, 10, 9);

        let (_, feed) = get_news_feed(&state, 1).await;

        assert_eq!(feed["home_page_url"], "https://novel.sis.jp");
        assert_eq!(
            feed["items"][0]["url"],
            "https://novel.sis.jp/novel/narou/n1234ab/10"
        );
    }

    #[tokio::test]
    async fn news_omits_date_published_when_unknown() {
        let state = test_state();
        insert_favorite(&state, 1, "kakuyomu", "k1", None, 10, 9);

        let (_, feed) = get_news_feed(&state, 1).await;
        let item = &feed["items"][0];
        assert!(
            item.get("date_published").is_none(),
            "date_published must be omitted (not null) when unknown"
        );
    }

    #[test]
    fn rfc3339_jst_tags_naive_timestamps() {
        assert_eq!(
            to_rfc3339_jst("2026-03-14T00:00:00"),
            Some("2026-03-14T00:00:00+09:00".to_string())
        );
        assert_eq!(
            to_rfc3339_jst("2026-03-14 12:34:56"),
            Some("2026-03-14T12:34:56+09:00".to_string()),
            "space separator must be normalized to T"
        );
    }

    #[test]
    fn rfc3339_jst_passes_through_zoned_timestamps() {
        assert_eq!(
            to_rfc3339_jst("2026-03-14T00:00:00Z"),
            Some("2026-03-14T00:00:00+00:00".to_string()),
            "Z is normalized to +00:00, preserving the instant"
        );
        assert_eq!(
            to_rfc3339_jst("2026-03-14T00:00:00+09:00"),
            Some("2026-03-14T00:00:00+09:00".to_string())
        );
        assert_eq!(
            to_rfc3339_jst("2026-03-14T00:00:00.123+09:00"),
            Some("2026-03-14T00:00:00.123+09:00".to_string()),
            "fractional seconds are valid RFC3339 and must survive"
        );
    }

    #[test]
    fn rfc3339_jst_rejects_malformed_input() {
        assert_eq!(to_rfc3339_jst(""), None);
        assert_eq!(to_rfc3339_jst("not a date"), None);
        assert_eq!(to_rfc3339_jst("2026-03-14"), None);
        assert_eq!(to_rfc3339_jst("2026-03-14T00:00:00junk"), None);
        assert_eq!(
            to_rfc3339_jst("2026-03-14T00:00:00+AB:CD"),
            None,
            "non-numeric offsets are not RFC3339"
        );
        assert_eq!(
            to_rfc3339_jst("更新日２０２６年"),
            None,
            "multi-byte garbage must not panic on byte slicing"
        );
    }
}
