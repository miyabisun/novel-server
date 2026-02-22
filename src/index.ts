import { serveStatic } from 'hono/bun'
import { Hono } from 'hono'
import { logger } from 'hono/logger'

import init from './lib/init.js'
import { startSync } from './lib/favorite-sync.js'
import ranking from './routes/ranking.js'
import pages from './routes/pages.js'
import detail from './routes/detail.js'
import favorites from './routes/favorites.js'
import search from './routes/search.js'
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
sub.route('/', search)

// Static files & SPA fallback
sub.use('/assets/*', serveStatic({
  root: './client/build',
  rewriteRequestPath: (p) => p.startsWith(basePath) ? p.slice(basePath.length) : p,
}))
sub.use('/favicon.svg', serveStatic({
  root: './client/build',
  rewriteRequestPath: () => '/favicon.svg',
}))

sub.get('*', (c) => {
  const html = getIndexHtml(basePath)
  if (html) return c.html(html)
  return c.json({ error: 'Frontend not built. Run: bun run build:client' }, 404)
})

if (basePath) {
  app.route(basePath, sub)
} else {
  app.route('/', sub)
}

await init()
startSync()

console.log(`Server running on http://localhost:${port}${basePath || '/'}`)

export default { port, fetch: app.fetch }
