import path from 'path'

const dbPath = process.env.DATABASE_PATH || '/data/novel.db'
const resolved = path.resolve(dbPath)

process.env.DATABASE_URL = `file:${resolved}`

console.log(`Database: ${resolved}`)
