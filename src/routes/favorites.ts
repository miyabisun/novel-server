import { Hono } from 'hono'
import { Prisma } from '@prisma/client'
import prisma from '../lib/prisma.js'
import M from '../modules/index.js'

const app = new Hono()
const VALID_TYPES = Object.keys(M)

function validateType(type: string): boolean {
  return VALID_TYPES.includes(type)
}

class NotFoundError extends Error {}

async function handleNotFound<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn()
  } catch (e) {
    if (e instanceof Prisma.PrismaClientKnownRequestError && e.code === 'P2025') {
      throw new NotFoundError()
    }
    throw e
  }
}

app.get('/api/favorites', async (c) => {
  const favorites = await prisma.favorite.findMany({ orderBy: { novelupdated_at: { sort: 'desc', nulls: 'last' } } })
  return c.json(favorites)
})

app.put('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  const body = await c.req.json()
  if (!body.title || body.page == null) {
    return c.json({ error: 'title and page are required' }, 400)
  }
  const favorite = await prisma.favorite.upsert({
    where: { type_id: { type, id } },
    update: { title: body.title, page: body.page, novelupdated_at: body.novelupdated_at ?? null },
    create: { type, id, title: body.title, page: body.page, novelupdated_at: body.novelupdated_at ?? null, read: 0 },
  })
  return c.json(favorite)
})

app.delete('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  try {
    await handleNotFound(() => prisma.favorite.delete({ where: { type_id: { type, id } } }))
    return c.json({ ok: true })
  } catch (e) {
    if (e instanceof NotFoundError) return c.json({ error: 'Not found' }, 404)
    throw e
  }
})

app.patch('/api/favorites/:type/:id/progress', async (c) => {
  const { type, id } = c.req.param()
  if (!validateType(type)) return c.json({ error: 'Invalid type' }, 400)
  const body = await c.req.json()
  if (body.read == null) return c.json({ error: 'read is required' }, 400)
  const existing = await prisma.favorite.findUnique({ where: { type_id: { type, id } } })
  if (!existing) return c.json({ ok: true })
  const favorite = await prisma.favorite.update({
    where: { type_id: { type, id } },
    data: { read: body.read },
  })
  return c.json(favorite)
})

export default app
