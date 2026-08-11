import { test, expect } from '@playwright/test'
import { mockApi } from './mock-api.js'

test.describe('400px chrome', () => {
  test.beforeEach(async ({ page }) => {
    await mockApi(page)
  })

  test('nav shows novel + 3-letter site labels and hamburger without overflow', async ({
    page,
  }) => {
    await page.goto('/')
    const header = page.locator('header')
    // Compact brand dual: root short is "novel"; desktop label "favorite" is hidden.
    await expect(header.getByRole('link', { name: 'novel' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'favorite' })).toHaveCount(0)
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

  test('ranking select focus rings stay fully inside the toolbar band', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('link', { name: 'nar' }).click()
    const toolbar = page.locator('.toolbar')
    const genre = page.locator('.genre-select')
    const period = page.locator('.period-select')
    await expect(toolbar).toBeVisible()
    await expect(genre).toBeVisible()
    await expect(period).toBeVisible()

    /**
     * Keyboard focus so :focus-visible applies (mouse click often does not).
     * Asserts the outline geometry fits inside the 40px toolbar (no clip under header).
     */
    async function assertFocusRingInsideBand(select) {
      await select.focus()
      // Re-trigger focus-visible via Tab cycle (Playwright focus() alone may not).
      await page.keyboard.down('Shift')
      await page.keyboard.press('Tab')
      await page.keyboard.up('Shift')
      await page.keyboard.press('Tab')

      const metrics = await select.evaluate((el) => {
        const cs = getComputedStyle(el)
        const r = el.getBoundingClientRect()
        const w = parseFloat(cs.outlineWidth) || 0
        const off = parseFloat(cs.outlineOffset) || 0
        return {
          focusVisible: el.matches(':focus-visible'),
          outlineStyle: cs.outlineStyle,
          outlineWidth: w,
          outlineOffset: off,
          ringTop: r.top - w - off,
          ringBottom: r.bottom + w + off,
        }
      })
      const band = await toolbar.evaluate((el) => {
        const r = el.getBoundingClientRect()
        return { top: r.top, bottom: r.bottom }
      })
      const headerBottom = await page
        .locator('header')
        .evaluate((el) => el.getBoundingClientRect().bottom)

      expect(metrics.focusVisible).toBe(true)
      expect(metrics.outlineStyle).not.toBe('none')
      expect(metrics.outlineWidth).toBe(2)
      expect(metrics.outlineOffset).toBe(-1)
      // Strict containment: ring must not enter the app header or leave the band.
      const floor = Math.max(band.top, headerBottom)
      expect(metrics.ringTop).toBeGreaterThanOrEqual(floor)
      expect(metrics.ringBottom).toBeLessThanOrEqual(band.bottom)
    }

    await assertFocusRingInsideBand(genre)
    await assertFocusRingInsideBand(period)
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
