import { test, expect } from '@playwright/test'
import { mockApi } from './mock-api.js'

// Page surfaces (--c-bg in global.sass): Kinari = light, Washi = e-paper, Sumi = dark.
const SURFACE = {
  kinari: 'rgb(250, 246, 239)',
  washi: 'rgb(250, 250, 250)',
  sumi: 'rgb(25, 25, 25)',
}

// The theme-color Chrome applies: the first meta[name=theme-color] in tree order
// whose media matches (HTML spec; Blink Document::ThemeColor).
function appliedThemeColor(page) {
  return page.evaluate(
    () =>
      [...document.querySelectorAll('meta[name="theme-color"]')].find(
        (meta) => !meta.media || matchMedia(meta.media).matches,
      )?.content,
  )
}

// Normalise a CSS colour to the computed rgb() form SURFACE uses.
function rgb(page, color) {
  return page.evaluate((value) => {
    const probe = document.createElement('i')
    probe.style.color = value
    document.body.append(probe)
    const computed = getComputedStyle(probe).color
    probe.remove()
    return computed
  }, color)
}

// In a browser tab theme-color carries the page background of the active theme.
async function expectTabThemeColor(page, surface) {
  await expect.poll(() => appliedThemeColor(page)).toBe(surface)
  expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe(surface)
}

// Rasterise an icon as a launcher would and measure, as fractions of the icon size, the
// pixels outside the safe zone (radius 40%) that differ from the corner background and the
// farthest artwork pixel from the centre.
function measureMaskableIcon(page, src) {
  return page.evaluate(async (url) => {
    const size = 512
    const image = new Image()
    image.src = url
    await image.decode()
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = size
    const context = canvas.getContext('2d')
    context.drawImage(image, 0, 0, size, size)
    const pixels = context.getImageData(0, 0, size, size).data
    const background = pixels.slice(0, 4)
    let outsideSafeZone = 0
    let artworkRadius = 0
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const i = (y * size + x) * 4
        const diff = Math.max(...[0, 1, 2, 3].map((c) => Math.abs(pixels[i + c] - background[c])))
        const radius = Math.hypot(x + 0.5 - size / 2, y + 0.5 - size / 2) / size
        if (radius > 0.4 && diff > 2) outsideSafeZone++
        if (diff > 32) artworkRadius = Math.max(artworkRadius, radius)
      }
    }
    return { opaque: background[3] === 255, outsideSafeZone, artworkRadius }
  }, src)
}

// Android Chrome 153-155 draws an installed app's system buttons over the window
// background, fixed at install from the manifest background_color (#F8F9FA without
// one), and picks their contrast from theme-color. The standalone theme-color must come
// first, so it wins there, and carry the Sumi backdrop the manifest declares.
async function expectInstalledAppThemeColor(page) {
  const first = page.locator('meta[name="theme-color"]').first()
  await expect(first).toHaveAttribute('media', '(display-mode: standalone)')
  expect(await rgb(page, await first.getAttribute('content'))).toBe(SURFACE.sumi)
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

  test('theme-color follows every theme choice in a tab and stays Sumi in an installed app', async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: 'light' })
    await page.goto('/')
    await expectTabThemeColor(page, SURFACE.kinari)
    await expectInstalledAppThemeColor(page)

    // 自動 follows the OS while the page stays open.
    await page.emulateMedia({ colorScheme: 'dark' })
    await expectTabThemeColor(page, SURFACE.sumi)

    await page.getByRole('button', { name: 'メニュー' }).click()
    await page.getByRole('navigation', { name: 'アプリメニュー' }).getByText('テーマ設定').click()
    for (const [label, surface] of [
      ['ライト', SURFACE.kinari],
      ['電子ペーパー', SURFACE.washi],
      ['ダーク', SURFACE.sumi],
    ]) {
      await page.getByRole('radio', { name: label }).click()
      await expectTabThemeColor(page, surface)
      await expectInstalledAppThemeColor(page)
    }

    // A fixed choice ignores the OS switch; 自動 picks the OS back up.
    await page.emulateMedia({ colorScheme: 'light' })
    await expectTabThemeColor(page, SURFACE.sumi)
    await page.getByRole('radio', { name: '自動' }).click()
    await expectTabThemeColor(page, SURFACE.kinari)

    await page.getByRole('radio', { name: '電子ペーパー' }).click()
    await page.reload()
    await expectTabThemeColor(page, SURFACE.washi)
    await expectInstalledAppThemeColor(page)
  })

  test('installed app manifest declares the Sumi backdrop', async ({ page }) => {
    await page.goto('/')
    // The site sits behind Cloudflare Access; a manifest fetch omits cookies by default.
    await expect(page.locator('link[rel="manifest"]')).toHaveAttribute(
      'crossorigin',
      'use-credentials',
    )
    const cdp = await page.context().newCDPSession(page)
    const { errors, data } = await cdp.send('Page.getAppManifest')
    expect(errors).toEqual([])
    const manifest = JSON.parse(data)
    expect(manifest.display).toBe('standalone')
    expect(await rgb(page, manifest.background_color)).toBe(SURFACE.sumi)
    expect(await rgb(page, manifest.theme_color)).toBe(SURFACE.sumi)
  })
})

// Android builds an adaptive home screen icon only from a maskable icon; an "any" icon
// becomes a legacy icon that the launcher shrinks into a white circle. The maskable icon
// must paint the whole square and keep its artwork inside the safe zone, large enough to
// read like a native app icon.
test('installed app offers a full-bleed maskable icon', async ({ page }) => {
  await mockApi(page)
  await page.goto('/')

  const cdp = await page.context().newCDPSession(page)
  const { url, errors, data } = await cdp.send('Page.getAppManifest')
  expect(errors).toEqual([])
  const purposes = (icon) => (icon.purpose ?? 'any').split(' ')
  const { icons } = JSON.parse(data)
  expect(icons.some((icon) => purposes(icon).includes('any'))).toBe(true)
  const maskable = icons.find((icon) => purposes(icon).includes('maskable'))
  expect(maskable).toBeDefined()

  const src = new URL(maskable.src, url).href
  const response = await page.request.get(src)
  expect(response.headers()['content-type']).toContain('image/svg+xml')
  const icon = await measureMaskableIcon(page, src)
  expect(icon.opaque).toBe(true)
  expect(icon.outsideSafeZone).toBe(0)
  expect(icon.artworkRadius).toBeGreaterThanOrEqual(0.3)
  expect(icon.artworkRadius).toBeLessThanOrEqual(0.4)
})
