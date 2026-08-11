/** Shared API stubs for chrome e2e (mobile + desktop). */
export async function mockApi(page) {
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
