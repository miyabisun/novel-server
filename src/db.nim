import db_connector/db_sqlite
import std/logging

proc open*(path: string): DbConn =
  info "Database: " & path
  result = db_sqlite.open(path, "", "", "")

  result.exec(sql"PRAGMA journal_mode = WAL")
  result.exec(sql"PRAGMA synchronous = NORMAL")

  result.exec(sql"""
    CREATE TABLE IF NOT EXISTS favorites (
      type TEXT NOT NULL,
      id TEXT NOT NULL,
      title TEXT NOT NULL,
      novelupdated_at TEXT,
      page INTEGER NOT NULL,
      read INTEGER NOT NULL DEFAULT 0,
      PRIMARY KEY (type, id)
    )
  """)
  result.exec(sql"""
    CREATE INDEX IF NOT EXISTS idx_favorites_updated
      ON favorites (novelupdated_at DESC)
  """)
