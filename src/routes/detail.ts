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

  const cached = cache.get(key) as { title: string; synopsis: string; page: number } | null
  if (cached) return c.json(cached)

  for (let i = 0; i < 3; i++) {
    try {
      const detail = await M[type].fetchDetail(id)
      cache.set(key, detail, DETAIL_TTL)
      return c.json(detail)
    } catch (e) {
      console.error(`fetchDetail ${type}/${id} attempt ${i + 1} failed:`, e)
      if (i < 2) await new Promise(r => setTimeout(r, 500 * (i + 1)))
    }
  }
  return c.json({ error: 'Failed to fetch detail' }, 502)
})

export default app
