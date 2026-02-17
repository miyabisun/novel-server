import { Hono } from 'hono'
import cache from '../lib/cache.js'
import M from '../modules/index.js'

const app = new Hono()

const VALID_TYPES = Object.keys(M)
const DETAIL_TTL = 60 * 60 * 24 // 24 hours

app.get('/api/novel/:type/:id/detail', async (c) => {
  const { type, id } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:${id}:detail`

  let detail = cache.get(key) as { title: string; synopsis: string } | undefined
  if (!detail) {
    try {
      detail = await M[type].fetchDetail(id)
      cache.set(key, detail, DETAIL_TTL)
    } catch {
      return c.json({ error: 'Failed to fetch detail' }, 502)
    }
  }
  return c.json(detail)
})

export default app
