import { Hono } from 'hono'
import { Prisma } from '@prisma/client'
import prisma from '../lib/prisma.js'

const app = new Hono()

app.get('/api/favorites', async (c) => {
  const favorites = await prisma.favorite.findMany({
    orderBy: { title: 'asc' },
  })
  return c.json(favorites)
})

app.put('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  const body = await c.req.json()
  if (!body.title || body.page == null) {
    return c.json({ error: 'title and page are required' }, 400)
  }
  const favorite = await prisma.favorite.upsert({
    where: { type_id: { type, id } },
    update: {
      title: body.title,
      page: body.page,
      novelupdated_at: body.novelupdated_at ?? null,
    },
    create: {
      type,
      id,
      title: body.title,
      page: body.page,
      novelupdated_at: body.novelupdated_at ?? null,
      read: 0,
    },
  })
  return c.json(favorite)
})

app.delete('/api/favorites/:type/:id', async (c) => {
  const { type, id } = c.req.param()
  try {
    await prisma.favorite.delete({
      where: { type_id: { type, id } },
    })
    return c.json({ ok: true })
  } catch (e) {
    if (e instanceof Prisma.PrismaClientKnownRequestError && e.code === 'P2025') {
      return c.json({ error: 'Not found' }, 404)
    }
    throw e
  }
})

app.patch('/api/favorites/:type/:id/progress', async (c) => {
  const { type, id } = c.req.param()
  const body = await c.req.json()
  if (body.read == null) {
    return c.json({ error: 'read is required' }, 400)
  }
  try {
    const favorite = await prisma.favorite.update({
      where: { type_id: { type, id } },
      data: { read: body.read },
    })
    return c.json(favorite)
  } catch (e) {
    if (e instanceof Prisma.PrismaClientKnownRequestError && e.code === 'P2025') {
      return c.json({ error: 'Not found' }, 404)
    }
    throw e
  }
})

export default app
