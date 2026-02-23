use crate::modules::ModuleType;
use crate::state::AppState;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    let mut stmt = match conn.prepare("SELECT id FROM favorites WHERE type = ?1") {
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

/// Update a single favorite record with fetched datum
pub fn update_favorite_from_datum(
    db: &Arc<Mutex<Connection>>,
    type_str: &str,
    datum: &Value,
) {
    let id = datum["id"].as_str().unwrap_or_default();
    let title = datum["title"].as_str().map(|s| s.to_string());
    let page = datum["pages"].as_array().map(|a| a.len() as i64);
    let novelupdated_at = datum["novelupdated_at"].as_str().map(|s| s.to_string());

    let conn = db.lock().unwrap();
    // Build dynamic UPDATE
    let mut sets = Vec::new();
    let mut param_idx = 1u32;
    let mut title_idx = 0u32;
    let mut page_idx = 0u32;
    let mut nua_idx = 0u32;

    if title.is_some() {
        title_idx = param_idx;
        sets.push(format!("title = ?{}", param_idx));
        param_idx += 1;
    }
    if page.is_some() {
        page_idx = param_idx;
        sets.push(format!("page = ?{}", param_idx));
        param_idx += 1;
    }
    if novelupdated_at.is_some() {
        nua_idx = param_idx;
        sets.push(format!("novelupdated_at = ?{}", param_idx));
        param_idx += 1;
    }

    if sets.is_empty() {
        return;
    }

    let sql = format!(
        "UPDATE favorites SET {} WHERE type = ?{} AND id = ?{}",
        sets.join(", "),
        param_idx,
        param_idx + 1
    );

    // Build params vector with concrete types using rusqlite::types::Value
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    // Fill in order of param_idx
    let total = param_idx + 1; // last idx for id
    for i in 1..=total {
        if i == title_idx {
            params.push(rusqlite::types::Value::Text(title.clone().unwrap()));
        } else if i == page_idx {
            params.push(rusqlite::types::Value::Integer(page.unwrap()));
        } else if i == nua_idx {
            params.push(rusqlite::types::Value::Text(novelupdated_at.clone().unwrap()));
        } else if i == param_idx {
            params.push(rusqlite::types::Value::Text(type_str.to_string()));
        } else if i == param_idx + 1 {
            params.push(rusqlite::types::Value::Text(id.to_string()));
        }
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
    let _ = conn.execute(&sql, param_refs.as_slice());
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
                // Use direct SQL within transaction scope
                for datum in &data {
                    let id = datum["id"].as_str().unwrap_or_default();
                    let title = datum["title"].as_str();
                    let page = datum["pages"].as_array().map(|a| a.len() as i64);
                    let novelupdated_at = datum["novelupdated_at"].as_str();

                    // Simple approach: always update all three fields
                    if title.is_some() || page.is_some() || novelupdated_at.is_some() {
                        // Use a single UPDATE with COALESCE-like approach
                        let _ = tx.execute(
                            "UPDATE favorites SET
                                title = COALESCE(?1, title),
                                page = COALESCE(?2, page),
                                novelupdated_at = COALESCE(?3, novelupdated_at)
                             WHERE type = ?4 AND id = ?5",
                            rusqlite::params![title, page, novelupdated_at, type_str, id],
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
                    tracing::info!(
                        "[sync] kakuyomu: updated {} ({}/{})",
                        id,
                        index + 1,
                        count
                    );
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
