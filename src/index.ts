import { serve } from '@hono/node-server'
import { serveStatic } from '@hono/node-server/serve-static'
import { Hono } from 'hono'
import { logger } from 'hono/logger'

import ranking from './routes/ranking.js'
import pages from './routes/pages.js'
import detail from './routes/detail.js'
import favorites from './routes/favorites.js'
import { getIndexHtml } from './lib/spa.js'

const port = Number(process.env.PORT) || 3000
const basePath = (process.env.BASE_PATH || '').replace(/\/+$/, '')
if (basePath && !/^\/[\w\-\/]*$/.test(basePath)) {
  throw new Error(`Invalid BASE_PATH: ${basePath}`)
}

const app = new Hono()
app.use('*', logger())

// Mount all routes under basePath
const sub = new Hono()

// API routes
sub.route('/', ranking)
sub.route('/', pages)
sub.route('/', detail)
sub.route('/', favorites)

// Static files & SPA fallback
sub.use('/assets/*', serveStatic({
  root: './client/build',
  rewriteRequestPath: (p) => p.startsWith(basePath) ? p.slice(basePath.length) : p,
}))

sub.get('*', (c) => {
  const html = getIndexHtml(basePath)
  if (html) return c.html(html)
  return c.json({ error: 'Frontend not built. Run: npm run build:client' }, 404)
})

if (basePath) {
  app.route(basePath, sub)
} else {
  app.route('/', sub)
}

console.log(`Server running on http://localhost:${port}${basePath || '/'}`)
serve({ fetch: app.fetch, port })
