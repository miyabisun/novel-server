import { Database } from 'bun:sqlite'
import { drizzle } from 'drizzle-orm/bun-sqlite'
import * as schema from './schema.js'

const dbPath = process.env.DATABASE_PATH || '/data/novel.db'
console.log(`Database: ${dbPath}`)

const sqlite = new Database(dbPath)
sqlite.exec('PRAGMA journal_mode = WAL')
sqlite.exec('PRAGMA synchronous = NORMAL')
sqlite.exec('PRAGMA cache_size = -64000')
sqlite.exec('PRAGMA temp_store = MEMORY')

export const db = drizzle(sqlite, { schema })
