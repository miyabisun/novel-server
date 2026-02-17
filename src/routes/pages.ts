import { Hono } from 'hono'
import * as cheerio from 'cheerio'
import cache from '../lib/cache.js'
import M from '../modules/index.js'

const app = new Hono()

const VALID_TYPES = Object.keys(M)
const PAGE_TTL = 60 * 60 * 24 // 24 hours

// cheerio.load() が生成する構造タグ + コンテンツ許可タグ
const ALLOWED_TAGS = new Set([
  'html', 'head', 'body',
  'p', 'br', 'hr', 'div', 'span',
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'ruby', 'rt', 'rp', 'rb',
  'em', 'strong', 'b', 'i', 'u', 's', 'sub', 'sup',
])

function sanitizeHtml(html: string | null): string {
  if (!html) return ''
  const $ = cheerio.load(html, { xml: false })
  // Repeat until no disallowed tags remain (handles nested unwrapping)
  let dirty = true
  while (dirty) {
    dirty = false
    $('*').each((_i, el) => {
      if (el.type !== 'tag') return
      if (!ALLOWED_TAGS.has(el.tagName)) {
        $(el).replaceWith($(el).contents())
        dirty = true
      } else {
        for (const attr of Object.keys(el.attribs)) {
          $(el).removeAttr(attr)
        }
      }
    })
  }
  return $('body').html() ?? ''
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
