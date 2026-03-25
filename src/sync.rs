use crate::modules::ModuleType;
use crate::state::AppState;
use chrono::Utc;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Periodically sync favorite metadata in the background.
///
/// - narou / nocturne: Bulk API fetch supports multiple IDs, so a fixed interval (10 min) suffices.
/// - kakuyomu: HTML scraping fetches one at a time, so sleep(3,600,000ms / count)
///   distributes requests evenly over 1 hour.
pub fn start_sync(state: AppState) {
    tracing::info!("[sync] starting background sync");
    start_syosetu_sync(state.clone(), ModuleType::Narou, Duration::from_secs(600));
    start_syosetu_sync(
        state.clone(),
        ModuleType::Nocturne,
        Duration::from_secs(600),
    );
    start_kakuyomu_sync(state);
}

fn get_ids(db: &Arc<Mutex<Connection>>, type_str: &str) -> Vec<String> {
    let conn = db.lock().unwrap();
    let mut stmt = match conn.prepare("SELECT DISTINCT id FROM favorites WHERE type = ?1") {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("[sync] {} db error: {}", type_str, e);
            return Vec::new();
        }
    };
    let result = match stmt.query_map(rusqlite::params![type_str], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            tracing::error!("[sync] {} query error: {}", type_str, e);
            Vec::new()
        }
    };
    result
}

/// Update a single favorite record with fetched datum.
/// Only updates `novelupdated_at` when `page` has increased (new chapters detected).
pub fn update_favorite_from_datum(db: &Arc<Mutex<Connection>>, type_str: &str, datum: &Value) {
    let id = datum["id"].as_str().unwrap_or_default();
    let title = datum["title"].as_str();
    let new_page = datum["pages"].as_array().map(|a| a.len() as i64);

    if title.is_none() && new_page.is_none() {
        return;
    }

    let conn = db.lock().unwrap();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "UPDATE favorites SET
            title = COALESCE(?1, title),
            page = COALESCE(?2, page),
            novelupdated_at = CASE WHEN ?2 > page THEN ?3 ELSE novelupdated_at END
         WHERE type = ?4 AND id = ?5",
        rusqlite::params![title, new_page, now, type_str, id],
    );
}

fn start_syosetu_sync(state: AppState, module: ModuleType, interval: Duration) {
    let type_str = module.as_str().to_string();
    tokio::spawn(async move {
        // Initial sync
        sync_syosetu(&state, &module, &type_str).await;

        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await; // skip immediate tick
        loop {
            ticker.tick().await;
            sync_syosetu(&state, &module, &type_str).await;
        }
    });
}

async fn sync_syosetu(state: &AppState, module: &ModuleType, type_str: &str) {
    let ids = get_ids(&state.db, type_str);
    if ids.is_empty() {
        return;
    }

    match module.fetch_data(&state.http, &ids).await {
        Ok(data) => {
            {
                let conn = state.db.lock().unwrap();
                let tx = match conn.unchecked_transaction() {
                    Ok(tx) => tx,
                    Err(e) => {
                        tracing::error!("[sync] {} transaction error: {}", type_str, e);
                        return;
                    }
                };
                let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                for datum in &data {
                    let id = datum["id"].as_str().unwrap_or_default();
                    let title = datum["title"].as_str();
                    let new_page = datum["pages"].as_array().map(|a| a.len() as i64);

                    if title.is_some() || new_page.is_some() {
                        let _ = tx.execute(
                            "UPDATE favorites SET
                                title = COALESCE(?1, title),
                                page = COALESCE(?2, page),
                                novelupdated_at = CASE WHEN ?2 > page THEN ?3 ELSE novelupdated_at END
                             WHERE type = ?4 AND id = ?5",
                            rusqlite::params![title, new_page, now, type_str, id],
                        );
                    }
                }
                let _ = tx.commit();
            }
            tracing::info!("[sync] {}: updated {} items", type_str, data.len());
        }
        Err(e) => {
            tracing::error!("[sync] {} error: {}", type_str, e);
        }
    }
}

fn start_kakuyomu_sync(state: AppState) {
    tokio::spawn(async move {
        let module = ModuleType::Kakuyomu;
        let type_str = "kakuyomu";
        let mut index: usize = 0;

        loop {
            let ids = get_ids(&state.db, type_str);
            let count = ids.len();
            if count == 0 {
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            index %= count;
            let id = ids[index].clone();

            match module.fetch_datum(&state.http, &id).await {
                Ok(datum) => {
                    update_favorite_from_datum(&state.db, type_str, &datum);
                    tracing::info!("[sync] kakuyomu: updated {} ({}/{})", id, index + 1, count);
                    index += 1;
                    let interval_ms = 3_600_000u64 / count as u64;
                    tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                }
                Err(e) => {
                    tracing::error!("[sync] kakuyomu error: {}", e);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        }
    });
}
