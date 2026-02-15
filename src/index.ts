import fs from 'fs'
import path from 'path'
import { serve } from '@hono/node-server'
import { serveStatic } from '@hono/node-server/serve-static'
import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { logger } from 'hono/logger'

import ranking from './routes/ranking.js'
import pages from './routes/pages.js'

const port = Number(process.env.PORT) || 3000
const basePath = (process.env.BASE_PATH || '').replace(/\/+$/, '')
if (basePath && !/^\/[\w\-\/]*$/.test(basePath)) {
  throw new Error(`Invalid BASE_PATH: ${basePath}`)
}

const app = new Hono()

// Middleware
app.use('*', cors())
app.use('*', logger())

// Mount all routes under basePath
const sub = new Hono()

// API routes
sub.route('/', ranking)
sub.route('/', pages)

// Serve built frontend static files
sub.use('/assets/*', serveStatic({
  root: './client/build',
  rewriteRequestPath: (p) => p.startsWith(basePath) ? p.slice(basePath.length) : p,
}))

// SPA fallback — read index.html once at startup
const indexPath = path.join(process.cwd(), 'client/build/index.html')
const indexHtml = fs.existsSync(indexPath)
  ? fs.readFileSync(indexPath, 'utf-8')
      .replace('<head>', `<head>\n\t\t<base href="${basePath}/">`)
      .replace('window.__BASE_PATH__ = ""', `window.__BASE_PATH__ = ${JSON.stringify(basePath)}`)
  : null

sub.get('*', (c) => {
  if (indexHtml) return c.html(indexHtml)
  return c.json({ error: 'Frontend not built. Run: npm run build:client' }, 404)
})

if (basePath) {
  app.route(basePath, sub)
} else {
  app.route('/', sub)
}

console.log(`Server running on http://localhost:${port}${basePath || '/'}`)
serve({ fetch: app.fetch, port })
