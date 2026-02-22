import * as cheerio from 'cheerio'

export function mapItem(obj: Record<string, unknown>): Record<string, unknown> {
  const acc: Record<string, unknown> = {}
  for (const [key, val] of Object.entries(obj)) {
    switch (key) {
      case 'ncode':
        acc['id'] = val != null ? String(val).toLowerCase() : val
        break
      case 'title':
        acc[key] = val != null ? String(val).trim() : val
        break
      case 'general_all_no':
        acc['page'] = val
        break
      default:
        acc[key] = val
    }
  }
  return acc
}

export function buildPages(type: string, id: string, count: number) {
  return Array.from({ length: count }, (_, i) => ({
    type,
    id,
    num: i + 1,
    page_id: String(i + 1),
  }))
}

export function createFetchApi(apiUrl: string) {
  return function fetchApi(q: Record<string, unknown>) {
    const params: Record<string, string> = { out: 'json' }
    for (const [key, val] of Object.entries(q)) {
      params[key] = Array.isArray(val) ? val.join('-') : String(val)
    }
    return fetch(`${apiUrl}?${new URLSearchParams(params)}`)
      .then((res) => {
        if (!res.ok) throw new Error(`API error: ${res.status}`)
        return res.json()
      })
      .then((json: any[]) => json.slice(1).map(mapItem))
  }
}

export function parseToc(html: string): { title: string; episodes: { num: number; title: string }[]; lastPage: number } {
  const $ = cheerio.load(html)
  const title = $('.p-novel__title').first().text().trim() || $('title').first().text().trim()
  const episodes: { num: number; title: string }[] = []
  let num = 0
  $('.p-eplist__sublist').each((_i, el) => {
    num++
    const epTitle = $(el).find('a').first().text().trim()
    episodes.push({ num, title: epTitle })
  })
  let lastPage = 1
  $('a').each((_i, el) => {
    if ($(el).text().trim() === '最後へ') {
      const match = ($(el).attr('href') || '').match(/[?&]p=(\d+)/)
      if (match) lastPage = Number(match[1])
    }
  })
  return { title, episodes, lastPage }
}

export function parsePage(html: string, selector: string): string | null {
  const $ = cheerio.load(html)
  const parts: string[] = []
  $(selector).each((_i, el) => {
    const h = $(el).html()
    if (h?.trim()) parts.push(h)
  })
  return parts.length ? parts.join('<hr>') : null
}
