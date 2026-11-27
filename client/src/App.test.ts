// @vitest-environment jsdom
// The page in jsdom: the header and the theme choice, found by role and name and worked with
// events fired at them. The server answers from the stand-in below. What only a browser's
// engine shows (the 400px layout, the colours, the manifest and its icon) is tested in
// tests/page.rs.
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import App from './App.svelte'
import './global.sass'

const json = (body: unknown) =>
  new Response(JSON.stringify(body), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })

/** The server's answers: a guest with one favorite. */
function api(input: RequestInfo | URL) {
  const url = String(input)
  if (url.includes('/api/auth/me')) return Promise.resolve(json({ email: 'guest' }))
  if (url.includes('/api/favorites')) {
    return Promise.resolve(
      json([
        {
          type: 'narou',
          id: 'n1234ab',
          title: 'テスト小説',
          page: 10,
          read: 3,
          novelupdated_at: null,
        },
      ]),
    )
  }
  return Promise.resolve(json([]))
}

/** A window of `compact` width (799px or narrower), as the header asks it. */
function windowOf(compact: boolean) {
  vi.stubGlobal(
    'matchMedia',
    (query: string) =>
      ({
        matches: query === '(max-width: 799px)' && compact,
        media: query,
        addEventListener() {},
        removeEventListener() {},
      }) as unknown as MediaQueryList,
  )
}

beforeEach(() => {
  localStorage.clear()
  delete document.documentElement.dataset.theme
  vi.stubGlobal('fetch', vi.fn(api))
})

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

describe('desktop chrome (≥800px)', () => {
  it('root tab shows favorite; novel-server title and full site labels visible', async () => {
    windowOf(false)
    render(App)
    const header = within(screen.getByRole('banner'))
    const title = document.querySelector('header .title') as HTMLElement
    expect(title.textContent).toBe('novel-server')
    expect(getComputedStyle(title).display).not.toBe('none')
    expect(header.getByRole('link', { name: 'favorite' }).getAttribute('href')).toBe('/')
    expect(header.queryByRole('link', { name: 'novel' })).toBeNull()
    for (const site of ['narou', 'kakuyomu', 'nocturne']) {
      expect(header.getByRole('link', { name: site })).toBeTruthy()
    }
  })
})

describe('400px chrome', () => {
  it('hamburger first action is テーマ設定; four theme choices including e-paper', async () => {
    windowOf(true)
    render(App)
    await fireEvent.click(screen.getByRole('button', { name: 'メニュー' }))
    const menu = screen.getByRole('navigation', { name: 'アプリメニュー' })
    expect(within(menu).getAllByRole('button')[0].textContent).toBe('テーマ設定')

    await fireEvent.click(within(menu).getByRole('button', { name: 'テーマ設定' }))
    const dialog = screen.getByRole('dialog', { name: 'テーマ設定' })
    for (const choice of ['自動', 'ライト', 'ダーク', '電子ペーパー']) {
      expect(within(dialog).getByRole('radio', { name: choice })).toBeTruthy()
    }

    await fireEvent.click(within(dialog).getByRole('radio', { name: '電子ペーパー' }))
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe('e-paper'))
    expect(localStorage.getItem('novel-server:theme')).toBe('e-paper')
  })
})
