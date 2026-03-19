use rusqlite::Connection;

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        email TEXT NOT NULL UNIQUE
    );
    INSERT OR IGNORE INTO users (id, email) VALUES (1, 'guest');

    CREATE TABLE IF NOT EXISTS favorites (
        user_id INTEGER NOT NULL DEFAULT 1,
        type TEXT NOT NULL,
        id TEXT NOT NULL,
        title TEXT NOT NULL,
        novelupdated_at TEXT,
        page INTEGER NOT NULL,
        read INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (user_id, type, id),
        FOREIGN KEY (user_id) REFERENCES users(id)
    );
    CREATE INDEX IF NOT EXISTS idx_favorites_updated
        ON favorites (user_id, novelupdated_at DESC);
";

pub fn open(path: &str) -> Connection {
    tracing::info!("Database: {}", path);
    let conn = Connection::open(path).expect("Failed to open database");

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA foreign_keys = ON;",
    )
    .expect("Failed to set PRAGMA");

    migrate(&conn);

    conn
}

fn migrate(conn: &Connection) {
    let has_users: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'users'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if has_users {
        // New schema already in place
        conn.execute_batch(SCHEMA).expect("Failed to create tables");
        return;
    }

    // Check if old favorites table exists (needs migration)
    let has_old_favorites: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'favorites'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if has_old_favorites {
        tracing::info!("Migrating database: adding users table and user_id to favorites");
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE users (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 email TEXT NOT NULL UNIQUE
             );
             INSERT INTO users (id, email) VALUES (1, 'guest');

             ALTER TABLE favorites RENAME TO favorites_old;

             CREATE TABLE favorites (
                 user_id INTEGER NOT NULL DEFAULT 1,
                 type TEXT NOT NULL,
                 id TEXT NOT NULL,
                 title TEXT NOT NULL,
                 novelupdated_at TEXT,
                 page INTEGER NOT NULL,
                 read INTEGER NOT NULL DEFAULT 0,
                 PRIMARY KEY (user_id, type, id),
                 FOREIGN KEY (user_id) REFERENCES users(id)
             );
             CREATE INDEX idx_favorites_updated ON favorites (user_id, novelupdated_at DESC);

             INSERT INTO favorites (user_id, type, id, title, novelupdated_at, page, read)
                 SELECT 1, type, id, title, novelupdated_at, page, read FROM favorites_old;

             DROP TABLE favorites_old;
             COMMIT;",
        )
        .expect("Failed to migrate database");
    } else {
        // Fresh database
        conn.execute_batch(SCHEMA).expect("Failed to create tables");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn
    }

    #[test]
    fn schema_is_idempotent() {
        let conn = open_memory();
        conn.execute_batch(SCHEMA).unwrap();
    }

    #[test]
    fn guest_user_exists() {
        let conn = open_memory();
        let email: String = conn
            .query_row("SELECT email FROM users WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(email, "guest");
    }

    #[test]
    fn insert_and_select() {
        let conn = open_memory();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "narou", "n1234ab", "Test Novel", 100),
        )
        .unwrap();

        let (title, page): (String, i64) = conn
            .query_row(
                "SELECT title, page FROM favorites WHERE user_id = ?1 AND type = ?2 AND id = ?3",
                (1, "narou", "n1234ab"),
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(title, "Test Novel");
        assert_eq!(page, 100);
    }

    #[test]
    fn primary_key_is_user_type_and_id() {
        let conn = open_memory();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "narou", "n1234ab", "Novel 1", 10),
        )
        .unwrap();

        // Same user+type+id should conflict
        let result = conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "narou", "n1234ab", "Novel 1 dup", 20),
        );
        assert!(result.is_err());

        // Different user, same type+id is allowed
        conn.execute(
            "INSERT INTO users (id, email) VALUES (2, 'alice@example.com')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (2, "narou", "n1234ab", "Novel 1", 10),
        )
        .unwrap();
    }

    #[test]
    fn read_defaults_to_zero() {
        let conn = open_memory();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "narou", "n1", "Novel", 50),
        )
        .unwrap();

        let read: i64 = conn
            .query_row(
                "SELECT read FROM favorites WHERE user_id = 1 AND type = ?1 AND id = ?2",
                ("narou", "n1"),
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(read, 0);
    }

    #[test]
    fn novelupdated_at_is_nullable() {
        let conn = open_memory();
        conn.execute(
            "INSERT INTO favorites (user_id, type, id, title, page) VALUES (?1, ?2, ?3, ?4, ?5)",
            (1, "narou", "n1", "Novel", 50),
        )
        .unwrap();

        let updated: Option<String> = conn
            .query_row(
                "SELECT novelupdated_at FROM favorites WHERE user_id = 1 AND type = ?1 AND id = ?2",
                ("narou", "n1"),
                |row| row.get(0),
            )
            .unwrap();
        assert!(updated.is_none());
    }

    #[test]
    fn index_exists_for_novelupdated_at() {
        let conn = open_memory();
        let index_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'index' AND name = 'idx_favorites_updated'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);
    }

    #[test]
    fn migration_from_old_schema() {
        let conn = Connection::open_in_memory().unwrap();
        // Create old schema (no users table, no user_id in favorites)
        conn.execute_batch(
            "CREATE TABLE favorites (
                type TEXT NOT NULL,
                id TEXT NOT NULL,
                title TEXT NOT NULL,
                novelupdated_at TEXT,
                page INTEGER NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (type, id)
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO favorites (type, id, title, page, read) VALUES ('narou', 'n1', 'Novel 1', 10, 5)",
            [],
        )
        .unwrap();

        // Run migration
        migrate(&conn);

        // Verify users table exists with guest
        let email: String = conn
            .query_row("SELECT email FROM users WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(email, "guest");

        // Verify migrated data has user_id = 1
        let (user_id, title, read): (i64, String, i64) = conn
            .query_row(
                "SELECT user_id, title, read FROM favorites WHERE type = 'narou' AND id = 'n1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(user_id, 1);
        assert_eq!(title, "Novel 1");
        assert_eq!(read, 5);
    }
}
