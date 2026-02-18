import { Hono } from 'hono'
import { eq, and, sql } from 'drizzle-orm'
import { db } from '../db/index.js'
import { favorites } from '../db/schema.js'
import M from '../modules/index.js'

const app = new Hono()
const VALID_TYPES = Object.keys(M)

function validateType(type: string): boolean {
  return VALID_TYPES.includes(type)
}

app.get('/api/favorites', async (c) => {
  const rows = db.select().from(favorites).orderBy(sql`novelupdated_at desc nulls last`).all()
  return c.json(rows)
})

app.put('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  const body = await c.req.json()
  if (!body.title || body.page == null) {
    return c.json({ error: 'title and page are required' }, 400)
  }
  const favorite = db.insert(favorites)
    .values({ type, id, title: body.title, page: body.page, novelupdated_at: body.novelupdated_at ?? null, read: 0 })
    .onConflictDoUpdate({
      target: [favorites.type, favorites.id],
      set: { title: body.title, page: body.page, novelupdated_at: body.novelupdated_at ?? null },
    })
    .returning()
    .get()
  return c.json(favorite)
})

app.delete('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  const deleted = db.delete(favorites)
    .where(and(eq(favorites.type, type), eq(favorites.id, id)))
    .returning()
    .get()
  if (!deleted) return c.json({ error: 'Not found' }, 404)
  return c.json({ ok: true })
})

app.patch('/api/favorites/:type/:id/progress', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  const body = await c.req.json()
  if (body.read == null) return c.json({ error: 'read is required' }, 400)
  const existing = db.select().from(favorites)
    .where(and(eq(favorites.type, type), eq(favorites.id, id)))
    .get()
  if (!existing) return c.json({ ok: true })
  const favorite = db.update(favorites)
    .set({ read: body.read })
    .where(and(eq(favorites.type, type), eq(favorites.id, id)))
    .returning()
    .get()
  return c.json(favorite)
})

export default app
