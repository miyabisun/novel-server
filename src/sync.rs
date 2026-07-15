use crate::modules::ModuleType;
use crate::state::AppState;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Refresh a favorite's metadata from a fetched datum. Shared by the periodic
/// sync and the fire-and-forget fetch on registration so the two never diverge.
///
/// `novelupdated_at` is set from the source API's "last episode published" time
/// (narou/nocturne `general_lastup`, kakuyomu `lastEpisodePublishedAt`, both
/// normalized to the `novelupdated_at` key). It therefore advances only when a
/// new chapter is posted — not on author edits to existing text — yet always
/// reflects the real publication time rather than a crawl-time approximation, so
/// even long-stale novels get a correct, non-NULL sort key.
///
/// The WHERE clause writes only when an API value actually differs from the
/// stored one (null-safe `IS NOT`), so re-running against unchanged novels is a
/// no-op and the periodic sync's "changed" count stays meaningful.
///
/// Params: ?1 = title, ?2 = page, ?3 = novelupdated_at, ?4 = type, ?5 = id.
const REFRESH_FAVORITE_SQL: &str = "UPDATE favorites SET
        title = COALESCE(?1, title),
        page = COALESCE(?2, page),
        novelupdated_at = COALESCE(?3, novelupdated_at)
     WHERE type = ?4 AND id = ?5
       AND ((?1 IS NOT NULL AND ?1 IS NOT title)
            OR (?2 IS NOT NULL AND ?2 IS NOT page)
            OR (?3 IS NOT NULL AND ?3 IS NOT novelupdated_at))";

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

/// Apply one fetched datum to its favorite row, returning the number of rows
/// actually changed (0 when nothing differed). Shared by the batched periodic
/// sync (called with a `Transaction`, which derefs to `Connection`) and the
/// single-row path below. See [`REFRESH_FAVORITE_SQL`] for the semantics.
fn refresh_favorite(conn: &Connection, type_str: &str, datum: &Value) -> usize {
    let id = datum["id"].as_str().unwrap_or_default();
    let title = datum["title"].as_str();
    let new_page = datum["pages"].as_array().map(|a| a.len() as i64);
    let novelupdated_at = datum["novelupdated_at"].as_str();

    if title.is_none() && new_page.is_none() && novelupdated_at.is_none() {
        return 0;
    }
    conn.execute(
        REFRESH_FAVORITE_SQL,
        rusqlite::params![title, new_page, novelupdated_at, type_str, id],
    )
    .unwrap_or(0)
}

/// Refresh a single favorite from a fetched datum (kakuyomu periodic sync and the
/// initial fetch on registration). See [`REFRESH_FAVORITE_SQL`] for the semantics.
pub fn update_favorite_from_datum(db: &Arc<Mutex<Connection>>, type_str: &str, datum: &Value) {
    let conn = db.lock().unwrap();
    refresh_favorite(&conn, type_str, datum);
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
            let mut changed = 0usize;
            {
                let conn = state.db.lock().unwrap();
                let tx = match conn.unchecked_transaction() {
                    Ok(tx) => tx,
                    Err(e) => {
                        tracing::error!("[sync] {} transaction error: {}", type_str, e);
                        return;
                    }
                };
                for datum in &data {
                    changed += refresh_favorite(&tx, type_str, datum);
                }
                let _ = tx.commit();
            }
            tracing::info!(
                "[sync] {}: checked {} items, {} changed",
                type_str,
                data.len(),
                changed
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn db_with_favorite(
        novelupdated_at: Option<&str>,
        page: i64,
        title: &str,
    ) -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::db::SCHEMA).unwrap();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, novelupdated_at, page)
             VALUES (1, 'narou', 'n1234ab', ?1, ?2, ?3)",
            rusqlite::params![title, novelupdated_at, page],
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn pages(n: usize) -> Value {
        Value::Array((0..n).map(|_| json!({})).collect())
    }

    fn stored(db: &Arc<Mutex<Connection>>) -> (String, Option<String>) {
        db.lock()
            .unwrap()
            .query_row(
                "SELECT title, novelupdated_at FROM favorites WHERE type = 'narou' AND id = 'n1234ab'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap()
    }

    /// A fetched datum with general_lastup already normalized to `novelupdated_at`.
    fn datum(title: &str, page_count: usize, novelupdated_at: Option<&str>) -> Value {
        let mut d = json!({ "id": "n1234ab", "title": title, "pages": pages(page_count) });
        if let Some(t) = novelupdated_at {
            d["novelupdated_at"] = json!(t);
        }
        d
    }

    #[test]
    fn seeds_novelupdated_at_on_registration_when_title_and_page_unchanged() {
        // Stale-novel registration: the client already stored the correct
        // title/page, leaving novelupdated_at NULL. The fetch must still seed it
        // from general_lastup even though title/page did not change. (Regression:
        // the old change-detection guard skipped unchanged rows, so the timestamp
        // stayed NULL and the favorite sank to the bottom of the list.)
        let db = db_with_favorite(None, 3, "Stale Novel");
        update_favorite_from_datum(
            &db,
            "narou",
            &datum("Stale Novel", 3, Some("2024-03-15 10:00:00")),
        );
        assert_eq!(stored(&db).1.as_deref(), Some("2024-03-15 10:00:00"));
    }

    #[test]
    fn overwrites_existing_value_with_api_general_lastup() {
        // A row previously stamped with a (now-removed) crawl time is corrected to
        // the real publication time on the next sync.
        let db = db_with_favorite(Some("2026-05-20 10:00:00"), 3, "Novel");
        update_favorite_from_datum(
            &db,
            "narou",
            &datum("Novel", 3, Some("2026-05-20 09:55:00")),
        );
        assert_eq!(stored(&db).1.as_deref(), Some("2026-05-20 09:55:00"));
    }

    #[test]
    fn keeps_existing_when_datum_has_no_novelupdated_at() {
        // kakuyomu omits the timestamp when lastEpisodePublishedAt is missing;
        // COALESCE must not clobber an existing value with NULL.
        let db = db_with_favorite(Some("2024-01-01 00:00:00"), 3, "Novel");
        update_favorite_from_datum(&db, "narou", &datum("Novel", 3, None));
        assert_eq!(stored(&db).1.as_deref(), Some("2024-01-01 00:00:00"));
    }

    #[test]
    fn refreshes_title_while_preserving_timestamp() {
        // A title change triggers the UPDATE, but a datum without novelupdated_at
        // must leave the stored timestamp untouched (independent COALESCE columns).
        let db = db_with_favorite(Some("2024-01-01 00:00:00"), 3, "Old Title");
        update_favorite_from_datum(&db, "narou", &datum("New Title", 3, None));
        let (title, ts) = stored(&db);
        assert_eq!(title, "New Title");
        assert_eq!(ts.as_deref(), Some("2024-01-01 00:00:00"));
    }
}
