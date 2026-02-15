import { Hono } from 'hono'
import cache from '../lib/cache.js'
import M from '../modules/index.js'

const app = new Hono()

const RANKING_TTL = 60 * 60 * 3 // 3 hours
const VALID_TYPES = Object.keys(M)

app.get('/api/novel/:type/ranking', async (c) => {
  const type = c.req.param('type')
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:ranking`

  let ranking = cache.get(key)
  if (!ranking) {
    try {
      ranking = await M[type].fetchRankingList()
      cache.set(key, ranking, RANKING_TTL)
    } catch {
      return c.json({ error: 'Failed to fetch ranking' }, 502)
    }
  }
  return c.json(ranking)
})

app.patch('/api/novel/:type/ranking', async (c) => {
  const type = c.req.param('type')
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)
  const key = `novel:${type}:ranking`

  try {
    const ranking = await M[type].fetchRankingList()
    cache.set(key, ranking, RANKING_TTL)
    return c.json(ranking)
  } catch {
    return c.json({ error: 'Failed to fetch ranking' }, 502)
  }
})

export default app
