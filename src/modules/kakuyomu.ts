import * as cheerio from 'cheerio'
import { format, parseISO } from 'date-fns'

const UA = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
const headers = { 'User-Agent': UA }
const type = 'kakuyomu'

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function parseApolloState(html: string): Record<string, any> {
  const $ = cheerio.load(html)
  const raw = $('#__NEXT_DATA__').text()
  if (!raw) throw new Error('Failed to parse kakuyomu work page')
  return JSON.parse(raw).props.pageProps.__APOLLO_STATE__
}

function extractWork(apollo: Record<string, any>) {
  const workKey = Object.keys(apollo).find((k) => k.startsWith('Work:'))
  if (!workKey) throw new Error('Work not found in Apollo state')
  const work = apollo[workKey]
  return {
    title: work.title as string,
    novelupdated_at: work.lastEpisodePublishedAt
      ? format(parseISO(work.lastEpisodePublishedAt), 'yyyy-MM-dd HH:mm:ss')
      : undefined,
  }
}

function extractEpisodes(apollo: Record<string, any>, id: string) {
  const pages: Record<string, unknown>[] = []
  const tocKeys = Object.keys(apollo).filter((k) => k.startsWith('TableOfContentsChapter'))
  let num = 0
  for (const tocKey of tocKeys) {
    for (const epRef of apollo[tocKey].episodeUnions || []) {
      const ref = epRef.__ref as string
      if (!ref || !apollo[ref]) continue
      const ep = apollo[ref]
      num++
      pages.push({ type, id, num, page_id: ep.id as string, title: ep.title as string })
    }
  }
  return pages
}

const kakuyomu = {
  async fetchRanking(genre: string, rankType: string) {
    const res = await fetch(`https://kakuyomu.jp/rankings/${genre}/${rankType}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu ranking error: ${res.status}`)
    const $ = cheerio.load(await res.text())
    const result: Record<string, unknown>[] = []
    $('.widget-work').each((_i, elem) => {
      const $e = $(elem)
      result.push({
        id: $e.find('.bookWalker-work-title').prop('href')?.replace(/.*\/(\d+)$/, '$1'),
        title: $e.find('.bookWalker-work-title').text(),
        page: Number($e.find('.widget-workCard-episodeCount').text().replace('話', '')),
      })
    })
    return result
  },

  async fetchRankingList(_limit?: number, period: string = 'daily') {
    if (period === 'quarter') throw new Error('kakuyomu does not support quarter ranking')
    return { '総合': await kakuyomu.fetchRanking('all', period) }
  },

  async fetchDatum(id: string) {
    const res = await fetch(`https://kakuyomu.jp/works/${id}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu work error: ${res.status}`)
    const apollo = parseApolloState(await res.text())
    const work = extractWork(apollo)
    const pages = extractEpisodes(apollo, id)
    return { type, id, ...work, pages }
  },

  async fetchData(ids: string[]) {
    const results: Record<string, unknown>[] = []
    for (const id of ids) {
      results.push(await kakuyomu.fetchDatum(id))
      await sleep(500)
    }
    return results
  },

  async fetchPage(id: string, pageId: string) {
    const res = await fetch(`https://kakuyomu.jp/works/${id}/episodes/${pageId}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu episode error: ${res.status}`)
    const $ = cheerio.load(await res.text())
    return $('.widget-episodeBody').html()
  },
}

export default kakuyomu
