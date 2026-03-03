use crate::error::AppError;
use crate::modules::ModuleType;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

fn map_favorite_row(row: &rusqlite::Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "type": row.get::<_, String>(0)?,
        "id": row.get::<_, String>(1)?,
        "title": row.get::<_, String>(2)?,
        "novelupdated_at": row.get::<_, Option<String>>(3)?,
        "page": row.get::<_, i64>(4)?,
        "read": row.get::<_, i64>(5)?,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/favorites", get(get_favorites))
        .route("/api/favorites/{type}/{id}", put(put_favorite))
        .route("/api/favorites/{type}/{id}", delete(delete_favorite))
        .route(
            "/api/favorites/{type}/{id}/progress",
            patch(patch_progress),
        )
}

#[derive(Deserialize)]
struct FavoriteBody {
    title: Option<String>,
    page: Option<i64>,
    novelupdated_at: Option<String>,
}

#[derive(Deserialize)]
struct ProgressBody {
    read: Option<i64>,
}

async fn get_favorites(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = {
        let db = state.db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT type, id, title, novelupdated_at, page, read FROM favorites ORDER BY novelupdated_at DESC NULLS LAST",
        )?;
        let rows = stmt
            .query_map([], map_favorite_row)?
            .collect::<Result<Vec<Value>, _>>()?;
        rows
    };
    Ok(Json(Value::Array(rows)))
}

async fn put_favorite(
    State(state): State<AppState>,
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
    let novelupdated_at = body.novelupdated_at;

    let favorite = {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO favorites (type, id, title, page, novelupdated_at, read) VALUES (?1, ?2, ?3, ?4, ?5, 0)
             ON CONFLICT(type, id) DO UPDATE SET title = ?3, page = ?4, novelupdated_at = ?5",
            rusqlite::params![type_str, id, title, page, novelupdated_at],
        )?;
        let mut stmt = db.prepare(
            "SELECT type, id, title, novelupdated_at, page, read FROM favorites WHERE type = ?1 AND id = ?2",
        )?;
        stmt.query_row(rusqlite::params![type_str, id], map_favorite_row)?
    };

    // Fire-and-forget: fetch metadata immediately after adding
    let state_clone = state.clone();
    let id_clone = id.clone();
    let type_clone = type_str.clone();
    tokio::spawn(async move {
        match module.fetch_datum(&state_clone.http, &id_clone).await {
            Ok(datum) => {
                crate::sync::update_favorite_from_datum(
                    &state_clone.db,
                    &type_clone,
                    &datum,
                );
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

async fn delete_favorite(
    State(state): State<AppState>,
    Path((type_str, id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    ModuleType::resolve(&type_str)?;
    let changes = {
        let db = state.db.lock().unwrap();
        db.execute(
            "DELETE FROM favorites WHERE type = ?1 AND id = ?2",
            rusqlite::params![type_str, id],
        )?
    };
    if changes == 0 {
        return Err(AppError::NotFound("Not found".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

async fn patch_progress(
    State(state): State<AppState>,
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
            "UPDATE favorites SET read = ?1 WHERE type = ?2 AND id = ?3",
            rusqlite::params![read, type_str, id],
        )?;
        if changes == 0 {
            return Ok(Json(json!({ "ok": true })));
        }
        let mut stmt = db.prepare(
            "SELECT type, id, title, novelupdated_at, page, read FROM favorites WHERE type = ?1 AND id = ?2",
        )?;
        stmt.query_row(rusqlite::params![type_str, id], map_favorite_row)?
    };
    Ok(Json(result))
}
