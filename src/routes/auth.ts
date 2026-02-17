import { timingSafeEqual } from 'node:crypto'
import { Hono } from 'hono'
import { sign } from 'hono/jwt'
import { setCookie, deleteCookie } from 'hono/cookie'

function safeEqual(a: string, b: string): boolean {
  const bufA = Buffer.from(a)
  const bufB = Buffer.from(b)
  if (bufA.length !== bufB.length) {
    timingSafeEqual(bufA, bufA) // constant time even on length mismatch
    return false
  }
  return timingSafeEqual(bufA, bufB)
}

const app = new Hono()

app.post('/api/auth/login', async (c) => {
  let body: { username?: string; password?: string }
  try {
    body = await c.req.json()
  } catch {
    return c.json({ error: 'Invalid request body' }, 400)
  }
  const { username, password } = body
  if (
    !username || !password ||
    !safeEqual(username, process.env.AUTH_USERNAME!) ||
    !safeEqual(password, process.env.AUTH_PASSWORD!)
  ) {
    return c.json({ error: 'Invalid credentials' }, 401)
  }

  const token = await sign(
    { sub: username, exp: Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 7 },
    process.env.JWT_SECRET!,
  )

  setCookie(c, 'novel_token', token, {
    httpOnly: true,
    sameSite: 'Lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7,
  })

  return c.json({ ok: true })
})

app.post('/api/auth/logout', (c) => {
  deleteCookie(c, 'novel_token', { path: '/' })
  return c.json({ ok: true })
})

app.get('/api/auth/me', (c) => {
  return c.json({ authenticated: true })
})

export default app
