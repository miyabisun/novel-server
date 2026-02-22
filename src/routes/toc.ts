import { Hono } from 'hono'
import M from '../modules/index.js'

const app = new Hono()

const VALID_TYPES = Object.keys(M)

app.get('/api/novel/:type/:id/toc', async (c) => {
  const { type, id } = c.req.param()
  if (!VALID_TYPES.includes(type)) return c.json({ error: 'Invalid type' }, 400)

  for (let i = 0; i < 3; i++) {
    try {
      const toc = await M[type].fetchToc(id)
      return c.json(toc)
    } catch (e) {
      console.error(`fetchToc ${type}/${id} attempt ${i + 1} failed:`, e)
      if (i < 2) await new Promise(r => setTimeout(r, 500 * (i + 1)))
    }
  }
  return c.json({ error: 'Failed to fetch table of contents' }, 502)
})

export default app
