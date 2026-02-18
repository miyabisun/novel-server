import prisma from './prisma.js'

export default async function init() {
  await prisma.$executeRaw`
    CREATE TABLE IF NOT EXISTS favorites (
      type TEXT NOT NULL,
      id TEXT NOT NULL,
      title TEXT NOT NULL,
      novelupdated_at TEXT,
      page INTEGER NOT NULL,
      read INTEGER NOT NULL DEFAULT 0,
      PRIMARY KEY (type, id)
    )
  `
}
