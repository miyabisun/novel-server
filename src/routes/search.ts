import { Hono } from 'hono'
import cache from '../lib/cache.js'
import M from '../modules/index.js'

const app = new Hono()

const SEARCH_TTL = 60 * 60 // 1 hour
const VALID_TYPES = Object.keys(M)

app.get('/api/novel/:type/search', async (c) => {
  const type = c.req.param('type')
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const q = c.req.query('q')?.trim()
  if (!q) return c.json({ error: 'Missing query parameter: q' }, 400)
  const key = `novel:${type}:search:${q}`

  let results = cache.get(key)
  if (!results) {
    try {
      results = await M[type].fetchSearch(q)
      cache.set(key, results, SEARCH_TTL)
    } catch {
      return c.json({ error: 'Failed to search' }, 502)
    }
  }
  return c.json(results)
})

export default app
