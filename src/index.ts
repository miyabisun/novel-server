import { serve } from '@hono/node-server'
import { serveStatic } from '@hono/node-server/serve-static'
import { Hono } from 'hono'
import { logger } from 'hono/logger'
import { jwt } from 'hono/jwt'
import { getCookie } from 'hono/cookie'

import ranking from './routes/ranking.js'
import pages from './routes/pages.js'
import detail from './routes/detail.js'
import auth from './routes/auth.js'
import favorites from './routes/favorites.js'
import { getIndexHtml } from './lib/spa.js'

// Validate required environment variables
const jwtSecret = process.env.JWT_SECRET
if (!jwtSecret) throw new Error('JWT_SECRET is not set')
if (!process.env.AUTH_USERNAME || !process.env.AUTH_PASSWORD) {
  throw new Error('AUTH_USERNAME and AUTH_PASSWORD must be set')
}

const port = Number(process.env.PORT) || 3000
const basePath = (process.env.BASE_PATH || '').replace(/\/+$/, '')
if (basePath && !/^\/[\w\-\/]*$/.test(basePath)) {
  throw new Error(`Invalid BASE_PATH: ${basePath}`)
}

const app = new Hono()
app.use('*', logger())

// Mount all routes under basePath
const sub = new Hono()

// JWT middleware for /api/* (skip login)
const jwtAuth = jwt({ secret: jwtSecret, cookie: 'novel_token', alg: 'HS256' })
sub.use('/api/*', async (c, next) => {
  // Safe: only reachable under basePath via sub-router mount
  if (c.req.path.endsWith('/api/auth/login')) return next()
  const token = getCookie(c, 'novel_token')
  if (!token) return c.json({ error: 'Unauthorized' }, 401)
  return jwtAuth(c, next)
})

// API routes
sub.route('/', auth)
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
