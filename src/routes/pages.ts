import { Hono } from 'hono'
import cache from '../lib/cache.js'
import { sanitizeHtml } from '../lib/sanitize.js'
import M from '../modules/index.js'

const app = new Hono()

const VALID_TYPES = Object.keys(M)
const PAGE_TTL = 60 * 60 * 24 // 24 hours

app.get('/api/novel/:type/:id/pages/:num', async (c) => {
  const { type, id, num } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:${id}:page:${num}`

  const cached = cache.get(key) as string | null
  if (cached !== null) return c.json({ html: cached })

  for (let i = 0; i < 3; i++) {
    try {
      const raw = await M[type].fetchPage(id, num)
      const html = sanitizeHtml(raw)
      cache.set(key, html, PAGE_TTL)
      return c.json({ html })
    } catch (e) {
      console.error(`fetchPage ${type}/${id}/${num} attempt ${i + 1} failed:`, e)
      if (i < 2) await new Promise(r => setTimeout(r, 500 * (i + 1)))
    }
  }
  return c.json({ error: 'Failed to fetch page' }, 502)
})

app.patch('/api/novel/:type/:id/pages/:num', async (c) => {
  const { type, id, num } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:${id}:page:${num}`

  for (let i = 0; i < 3; i++) {
    try {
      const raw = await M[type].fetchPage(id, num)
      const html = sanitizeHtml(raw)
      cache.set(key, html, PAGE_TTL)
      return c.json({ html })
    } catch (e) {
      console.error(`fetchPage PATCH ${type}/${id}/${num} attempt ${i + 1} failed:`, e)
      if (i < 2) await new Promise(r => setTimeout(r, 500 * (i + 1)))
    }
  }
  return c.json({ error: 'Failed to fetch page' }, 502)
})

export default app
