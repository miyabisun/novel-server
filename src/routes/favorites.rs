use crate::auth::UserId;
use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, put};
use axum::{Extension, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

fn map_favorite_row(row: &rusqlite::Row) -> rusqlite::Result<Value> {
    let novelupdated_at = row
        .get::<_, Option<i64>>(3)?
        .and_then(crate::time::unix_timestamp_to_rfc3339);
    Ok(json!({
        "type": row.get::<_, String>(0)?,
        "id": row.get::<_, String>(1)?,
        "title": row.get::<_, String>(2)?,
        "novelupdated_at": novelupdated_at,
        "page": row.get::<_, i64>(4)?,
        "read": row.get::<_, i64>(5)?,
    }))
}

fn find_favorite(
    db: &rusqlite::Connection,
    user_id: i64,
    type_str: &str,
    id: &str,
) -> rusqlite::Result<Value> {
    let mut stmt = db.prepare(
        "SELECT type, id, title, novelupdated_at, page, read FROM favorites
         WHERE user_id = ?1 AND type = ?2 AND id = ?3",
    )?;
    stmt.query_row(rusqlite::params![user_id, type_str, id], map_favorite_row)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/favorites", get(get_favorites))
        .route("/api/favorites/{type}/{id}", put(put_favorite))
        .route("/api/favorites/{type}/{id}", delete(delete_favorite))
        .route("/api/favorites/{type}/{id}/progress", patch(patch_progress))
}

#[derive(Deserialize)]
struct FavoriteBody {
    title: Option<String>,
    page: Option<i64>,
    novelupdated_at: Option<String>,
    read: Option<i64>,
}

#[derive(Deserialize)]
struct ProgressBody {
    read: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/favorites",
    tag = "お気に入り",
    summary = "お気に入り一覧取得",
    description = "お気に入りに登録された小説の一覧を取得する。小説更新日時の降順でソートされる（更新日時のないものは末尾）。キャッシュなし。",
    responses(
        (status = 200, description = "お気に入り一覧", body = Vec<crate::openapi::Favorite>,
            example = json!([{"type": "narou", "id": "n1234ab", "title": "小説タイトル", "novelupdated_at": "2026-02-15T00:00:00Z", "page": 150, "read": 42}])),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn get_favorites(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> Result<Json<Value>, AppError> {
    let rows = {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT type, id, title, novelupdated_at, page, read FROM favorites
             WHERE user_id = ?1 ORDER BY novelupdated_at DESC NULLS LAST",
        )?;
        let rows = stmt
            .query_map([user_id.0], map_favorite_row)?
            .collect::<Result<Vec<Value>, _>>()?;
        rows
    };
    Ok(Json(Value::Array(rows)))
}

#[utoipa::path(
    put,
    path = "/api/favorites/{type}/{id}",
    tag = "お気に入り",
    summary = "お気に入り登録・更新",
    description = "お気に入りを追加または更新する（UPSERT動作）。登録後、バックグラウンドで小説のメタデータを非同期取得し、タイトル・ページ数・更新日時を最新化する。",
    params(
        ("type" = String, Path, description = "対象サイト（narou / nocturne / kakuyomu）", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
    ),
    request_body(content = crate::openapi::FavoriteRequest, description = "お気に入り情報。novelupdated_atとreadは省略可。readは新規登録時の初期既読位置で、既存のお気に入りには適用されない",
        example = json!({"title": "小説タイトル", "page": 150, "novelupdated_at": "2026-02-15T09:00:00+09:00", "read": 3})),
    responses(
        (status = 200, description = "作成/更新されたお気に入り", body = crate::openapi::Favorite),
        (status = 400, description = "必須フィールド不足", body = crate::openapi::ErrorResponse,
            example = json!({"error": "title and page are required"})),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn put_favorite(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Path((type_str, id)): Path<(String, String)>,
    Json(body): Json<FavoriteBody>,
) -> Result<Json<Value>, AppError> {
    let module = ModuleType::resolve(&type_str)?;
    let title = body
        .title
        .ok_or_else(|| AppError::BadRequest("title and page are required".into()))?;
    let page = body
        .page
        .ok_or_else(|| AppError::BadRequest("title and page are required".into()))?;
    let novelupdated_at = match body.novelupdated_at.as_deref() {
        Some(value) => Some(crate::time::parse_upstream_timestamp(value).ok_or_else(|| {
            AppError::BadRequest("novelupdated_at must be RFC3339 or a JST date-time".into())
        })?),
        None => None,
    };

    let favorite = {
        let db = state.db.lock().unwrap();
        // `read` seeds only new rows; ON CONFLICT leaves an existing bookmark
        // untouched so a re-registration never rewinds reading progress.
        db.execute(
            "INSERT INTO favorites (user_id, type, id, title, page, novelupdated_at, read) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(user_id, type, id) DO UPDATE SET title = ?4, page = ?5, novelupdated_at = ?6",
            rusqlite::params![
                user_id.0,
                type_str,
                id,
                title,
                page,
                novelupdated_at,
                body.read.unwrap_or(0)
            ],
        )?;
        find_favorite(&db, user_id.0, &type_str, &id)?
    };

    // Fire-and-forget: fetch metadata immediately after adding
    let state_clone = state.clone();
    let id_clone = id.clone();
    let type_clone = type_str.clone();
    tokio::spawn(async move {
        match module.fetch_datum(&state_clone.http, &id_clone).await {
            Ok(datum) => {
                crate::sync::update_favorite_from_datum(&state_clone.db, &type_clone, &datum);
                tracing::info!("[sync] initial fetch for {}/{}", type_clone, id_clone);
            }
            Err(e) => {
                tracing::error!(
                    "[sync] initial fetch failed for {}/{}: {}",
                    type_clone,
                    id_clone,
                    e
                );
            }
        }
    });

    Ok(Json(favorite))
}

#[utoipa::path(
    delete,
    path = "/api/favorites/{type}/{id}",
    tag = "お気に入り",
    summary = "お気に入り削除",
    description = "お気に入りを削除する。",
    params(
        ("type" = String, Path, description = "対象サイト", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
    ),
    responses(
        (status = 200, description = "削除成功", body = crate::openapi::OkResponse,
            example = json!({"ok": true})),
        (status = 404, description = "お気に入りが存在しない", body = crate::openapi::ErrorResponse),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn delete_favorite(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Path((type_str, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    ModuleType::resolve(&type_str)?;
    let changes = {
        let db = state.db.lock().unwrap();
        db.execute(
            "DELETE FROM favorites WHERE user_id = ?1 AND type = ?2 AND id = ?3",
            rusqlite::params![user_id.0, type_str, id],
        )?
    };
    if changes == 0 {
        return Err(AppError::NotFound("Not found".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

#[utoipa::path(
    patch,
    path = "/api/favorites/{type}/{id}/progress",
    tag = "お気に入り",
    summary = "既読位置更新",
    description = "既読ページ位置を更新する。お気に入りに登録されていない場合は何もせず `{\"ok\": true}` を返す。",
    params(
        ("type" = String, Path, description = "対象サイト", example = "narou"),
        ("id" = String, Path, description = "小説ID", example = "n1234ab"),
    ),
    request_body(content = crate::openapi::ProgressRequest, description = "既読ページ番号",
        example = json!({"read": 42})),
    responses(
        (status = 200, description = "更新されたお気に入り（未登録の場合は {ok: true}）", body = crate::openapi::Favorite),
        (status = 400, description = "readフィールド不足", body = crate::openapi::ErrorResponse,
            example = json!({"error": "read is required"})),
        (status = 500, description = "DBエラー", body = crate::openapi::ErrorResponse),
    ),
)]
async fn patch_progress(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Path((type_str, id)): Path<(String, String)>,
    Json(body): Json<ProgressBody>,
) -> Result<Json<Value>, AppError> {
    ModuleType::resolve(&type_str)?;
    let read = body
        .read
        .ok_or_else(|| AppError::BadRequest("read is required".into()))?;

    let result = {
        let db = state.db.lock().unwrap();
        let changes = db.execute(
            "UPDATE favorites SET read = ?1 WHERE user_id = ?2 AND type = ?3 AND id = ?4",
            rusqlite::params![read, user_id.0, type_str, id],
        )?;
        if changes == 0 {
            return Ok(Json(json!({ "ok": true })));
        }
        find_favorite(&db, user_id.0, &type_str, &id)?
    };
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    // Bookmark seeding contract for PUT /api/favorites/{type}/{id}:
    // - `read` in the body seeds the bookmark of a NEW registration
    //   (register while reading episode N → bookmark at N)
    // - omitting `read` keeps the previous default of 0
    // - a PUT against an existing favorite never touches its bookmark,
    //   with or without `read` (no rewind on re-registration)

    use super::*;
    use crate::cache::Cache;
    use crate::config::Config;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let state = AppState {
            db: Arc::new(Mutex::new(crate::db::open(":memory:"))),
            cache: Arc::new(Cache::new()),
            config: Config {
                port: 3000,
                base_path: String::new(),
                public_base_url: None,
                db_path: String::new(),
            },
            http: reqwest::Client::new(),
        };
        state
            .db
            .lock()
            .unwrap()
            .execute(
                "INSERT OR IGNORE INTO users (id, email) VALUES (1, 'u1')",
                [],
            )
            .unwrap();
        state
    }

    async fn put_favorite_req(state: &AppState, id: &str, body: Value) -> (StatusCode, Value) {
        let app = Router::new()
            .route("/api/favorites/{type}/{id}", put(put_favorite))
            .layer(Extension(UserId(1)))
            .with_state(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/favorites/narou/{}", id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    fn stored_read(state: &AppState, id: &str) -> i64 {
        let db = state.db.lock().unwrap();
        db.query_row(
            "SELECT read FROM favorites WHERE user_id = 1 AND type = 'narou' AND id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn put_favorite_seeds_bookmark_on_new_registration() {
        let state = test_state();
        let (status, fav) = put_favorite_req(
            &state,
            "n_new",
            json!({"title": "T", "page": 10, "read": 3}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(fav["read"], 3);
        assert_eq!(stored_read(&state, "n_new"), 3);
    }

    #[tokio::test]
    async fn put_favorite_defaults_bookmark_to_zero() {
        let state = test_state();
        let (status, fav) =
            put_favorite_req(&state, "n_zero", json!({"title": "T", "page": 10})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(fav["read"], 0);
        assert_eq!(stored_read(&state, "n_zero"), 0);
    }

    #[tokio::test]
    async fn put_favorite_never_rewinds_existing_bookmark() {
        let state = test_state();
        put_favorite_req(
            &state,
            "n_keep",
            json!({"title": "T", "page": 10, "read": 3}),
        )
        .await;

        // Update without read keeps the bookmark
        let (_, fav) = put_favorite_req(&state, "n_keep", json!({"title": "T2", "page": 11})).await;
        assert_eq!(fav["read"], 3);

        // Update with a smaller read must not rewind either
        let (_, fav) = put_favorite_req(
            &state,
            "n_keep",
            json!({"title": "T3", "page": 12, "read": 1}),
        )
        .await;
        assert_eq!(fav["read"], 3);
        assert_eq!(stored_read(&state, "n_keep"), 3);
    }
}
