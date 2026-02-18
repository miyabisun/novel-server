import { sql } from 'drizzle-orm'
import { db } from '../db/index.js'

export default function init() {
  db.run(sql`
    CREATE TABLE IF NOT EXISTS favorites (
      type TEXT NOT NULL,
      id TEXT NOT NULL,
      title TEXT NOT NULL,
      novelupdated_at TEXT,
      page INTEGER NOT NULL,
      read INTEGER NOT NULL DEFAULT 0,
      PRIMARY KEY (type, id)
    )
  `)
}
