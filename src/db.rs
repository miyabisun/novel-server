use rusqlite::Connection;

pub fn open(path: &str) -> Connection {
    tracing::info!("Database: {}", path);
    let conn = Connection::open(path).expect("Failed to open database");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;",
    )
    .expect("Failed to set PRAGMA");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS favorites (
            type TEXT NOT NULL,
            id TEXT NOT NULL,
            title TEXT NOT NULL,
            novelupdated_at TEXT,
            page INTEGER NOT NULL,
            read INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (type, id)
        );
        CREATE INDEX IF NOT EXISTS idx_favorites_updated
            ON favorites (novelupdated_at DESC);",
    )
    .expect("Failed to create tables");

    conn
}
