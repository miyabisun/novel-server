import { test, expect } from '@playwright/test'

async function mockApi(page) {
  await page.route('**/api/**', async (route) => {
    const url = route.request().url()
    if (url.includes('/api/auth/me')) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ email: 'guest' }),
      })
    }
    if (url.includes('/api/favorites') && route.request().method() === 'GET') {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          {
            type: 'narou',
            id: 'n1234ab',
            title: 'テスト小説',
            page: 10,
            read: 3,
            novelupdated_at: null,
          },
        ]),
      })
    }
    if (url.includes('/ranking')) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          総合: [
            {
              id: 'n9999zz',
              title: 'ランキング透け確認用の長いタイトル文字列を並べる',
              page: 50,
              noveltype: 1,
            },
            {
              id: 'n8888yy',
              title: '二件目の小説タイトルでスクロールする',
              page: 12,
              noveltype: 1,
            },
          ],
        }),
      })
    }
    if (url.includes('/detail')) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ title: 'テスト小説', synopsis: 'あらすじ', page: 10 }),
      })
    }
    if (url.includes('/pages/')) {
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ html: '<p>本文です。スクロール用の段落。</p>'.repeat(20) }),
      })
    }
    return route.fulfill({ status: 200, contentType: 'application/json', body: '[]' })
  })
}

test.describe('400px chrome', () => {
  test.beforeEach(async ({ page }) => {
    await mockApi(page)
  })

  test('nav shows novel + 3-letter site labels and hamburger without overflow', async ({
    page,
  }) => {
    await page.goto('/')
    const header = page.locator('header')
    await expect(header.getByRole('link', { name: 'novel' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'nar' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'kak' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'noc' })).toBeVisible()
    await expect(header.getByRole('button', { name: 'メニュー' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'narou' })).toHaveCount(0)

    const overflow = await page.evaluate(() => {
      const root = document.documentElement
      return root.scrollWidth > root.clientWidth + 1
    })
    expect(overflow).toBe(false)
  })

  test('hamburger first action is テーマ設定; four theme choices including e-paper', async ({
    page,
  }) => {
    await page.goto('/')
    await page.getByRole('button', { name: 'メニュー' }).click()
    const menu = page.getByRole('navigation', { name: 'アプリメニュー' })
    await expect(menu).toBeVisible()
    const firstAction = menu.locator('button.menu-item').first()
    await expect(firstAction).toHaveText('テーマ設定')

    await firstAction.click()
    const dialog = page.getByRole('dialog', { name: 'テーマ設定' })
    await expect(dialog).toBeVisible()
    await expect(page.getByRole('radio', { name: '自動' })).toBeVisible()
    await expect(page.getByRole('radio', { name: 'ライト' })).toBeVisible()
    await expect(page.getByRole('radio', { name: 'ダーク' })).toBeVisible()
    await expect(page.getByRole('radio', { name: '電子ペーパー' })).toBeVisible()

    await page.getByRole('radio', { name: '電子ペーパー' }).click()
    await expect
      .poll(async () => page.evaluate(() => document.documentElement.dataset.theme))
      .toBe('e-paper')
    await expect
      .poll(async () => page.evaluate(() => localStorage.getItem('novel-server:theme')))
      .toBe('e-paper')
  })

  test('ranking toolbar is opaque surface band with 36px controls', async ({ page }) => {
    // Client-side navigation: vite preview + base './' may not SPA-fallback deep URLs.
    await page.goto('/')
    await page.getByRole('link', { name: 'nar' }).click()
    const toolbar = page.locator('.toolbar')
    await expect(toolbar).toBeVisible()
    const styles = await toolbar.evaluate((el) => {
      const cs = getComputedStyle(el)
      return {
        position: cs.position,
        top: cs.top,
        background: cs.backgroundColor,
        height: Math.round(parseFloat(cs.height)),
      }
    })
    expect(styles.position).toBe('sticky')
    expect(styles.top).toBe('48px')
    expect(styles.height).toBe(40)
    // --c-surface under Sumi (#232323) is opaque rgb
    expect(styles.background).toMatch(/rgb\(35,\s*35,\s*35\)/)

    const selectH = await page
      .locator('.genre-select')
      .evaluate((el) => Math.round(parseFloat(getComputedStyle(el).height)))
    expect(selectH).toBe(36)

    await page.evaluate(() => window.scrollTo(0, 200))
    await expect(toolbar).toBeVisible()
    const stillOpaque = await toolbar.evaluate((el) => {
      const cs = getComputedStyle(el)
      return cs.backgroundColor !== 'rgba(0, 0, 0, 0)' && cs.backgroundColor !== 'transparent'
    })
    expect(stillOpaque).toBe(true)
  })

  test('reader moves 目次 and unfav into hamburger on compact viewport', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('link', { name: 'テスト小説' }).click()
    await expect(page.locator('.reader-bar')).toBeVisible()
    await expect(page.locator('.reader-bar .toc-btn')).toBeHidden()
    await expect(page.locator('.reader-bar .fav-btn-remove')).toBeHidden()

    await page.getByRole('button', { name: 'メニュー' }).click()
    const menu = page.getByRole('navigation', { name: 'アプリメニュー' })
    const items = menu.locator('button.menu-item')
    await expect(items.nth(0)).toHaveText('テーマ設定')
    await expect(menu.getByRole('button', { name: '目次' })).toBeVisible()
    await expect(menu.getByRole('button', { name: 'お気に入りから削除' })).toBeVisible()
  })
})
