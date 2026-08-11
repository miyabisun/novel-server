import { test, expect } from '@playwright/test'
import { mockApi } from './mock-api.js'

test.describe('desktop chrome (≥800px)', () => {
  test.beforeEach(async ({ page }) => {
    await mockApi(page)
  })

  test('root tab shows favorite; novel-server title and full site labels visible', async ({
    page,
  }) => {
    await page.goto('/')
    const header = page.locator('header')
    await expect(header.locator('.title')).toHaveText('novel-server')
    await expect(header.locator('.title')).toBeVisible()
    await expect(header.getByRole('link', { name: 'favorite' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'novel' })).toHaveCount(0)
    await expect(header.getByRole('link', { name: 'narou' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'kakuyomu' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'nocturne' })).toBeVisible()
    await expect(header.getByRole('link', { name: 'favorite' })).toHaveAttribute('href', '/')
  })
})
