import { sqliteTable, text, integer, primaryKey } from 'drizzle-orm/sqlite-core'

export const favorites = sqliteTable('favorites', {
  type: text('type').notNull(),
  id: text('id').notNull(),
  title: text('title').notNull(),
  novelupdated_at: text('novelupdated_at'),
  page: integer('page').notNull(),
  read: integer('read').notNull().default(0),
}, (table) => [primaryKey({ columns: [table.type, table.id] })])
