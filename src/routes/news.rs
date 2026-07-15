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
    if let Some(dt) = item
        .novelupdated_at
        .and_then(crate::time::unix_timestamp_to_rfc3339)
    {
        obj["date_published"] = json!(dt);
    }
    obj
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
    // - date_published is RFC3339 UTC converted from the stored Unix timestamp
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
        updated_at: Option<i64>,
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
        insert_favorite(&state, 1, "narou", "n_done", Some(1_773_446_400), 10, 10);
        insert_favorite(&state, 1, "narou", "n_read", Some(1_773_360_000), 10, 5);
        insert_favorite(&state, 1, "narou", "n_far", Some(1_773_273_600), 30, 10);

        let (status, feed) = get_news_feed(&state, 1).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(item_ids(&feed), vec!["narou/n_read"]);
    }

    #[tokio::test]
    async fn news_sorted_by_updated_desc_nulls_last() {
        let state = test_state();
        insert_favorite(&state, 1, "narou", "n_old", Some(1_772_323_200), 10, 9);
        insert_favorite(&state, 1, "narou", "n_new", Some(1_773_446_400), 10, 9);
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
        insert_favorite(&state, 1, "narou", "n_u1", Some(1_773_446_400), 10, 9);
        insert_favorite(&state, 2, "narou", "n_u2", Some(1_773_446_400), 10, 9);

        let (_, feed1) = get_news_feed(&state, 1).await;
        assert_eq!(item_ids(&feed1), vec!["narou/n_u1"]);

        let (_, feed2) = get_news_feed(&state, 2).await;
        assert_eq!(item_ids(&feed2), vec!["narou/n_u2"]);
    }

    #[tokio::test]
    async fn news_items_follow_json_feed_1_1_with_news_extension() {
        let state = test_state();
        insert_favorite(&state, 1, "narou", "n1234ab", Some(1_773_446_400), 100, 98);

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
            item["date_published"], "2026-03-14T00:00:00Z",
            "stored Unix seconds must be exposed as RFC3339 UTC"
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
}
