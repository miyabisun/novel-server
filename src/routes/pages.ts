import { Hono } from 'hono'
import cache from '../lib/cache.js'
import M from '../modules/index.js'

const app = new Hono()

const VALID_TYPES = Object.keys(M)
const PAGE_TTL = 60 * 60 * 24 // 24 hours

function sanitizeHtml(html: string | null): string {
  if (!html) return ''
  return html
    .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, '')
    .replace(/<iframe\b[^>]*>[\s\S]*?<\/iframe>/gi, '')
    .replace(/\s+on\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]*)/gi, '')
}

app.get('/api/novel/:type/:id/pages/:num', async (c) => {
  const { type, id, num } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:${id}:page:${num}`

  let html = cache.get(key) as string | null
  if (!html) {
    try {
      const raw = await M[type].fetchPage(id, num)
      html = sanitizeHtml(raw)
      cache.set(key, html, PAGE_TTL)
    } catch {
      return c.json({ error: 'Failed to fetch page' }, 502)
    }
  }
  return c.json({ html })
})

app.patch('/api/novel/:type/:id/pages/:num', async (c) => {
  const { type, id, num } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:${id}:page:${num}`

  try {
    const raw = await M[type].fetchPage(id, num)
    const html = sanitizeHtml(raw)
    cache.set(key, html, PAGE_TTL)
    return c.json({ html })
  } catch {
    return c.json({ error: 'Failed to fetch page' }, 502)
  }
})

export default app
