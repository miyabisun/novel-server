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

function extractWork(apollo: Record<string, any>, id: string) {
  const work = apollo[`Work:${id}`]
  if (!work) throw new Error('Work not found in Apollo state')
  return {
    title: work.title as string,
    story: (work.introduction ?? '') as string,
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

  async fetchSearch(word: string) {
    const res = await fetch(`https://kakuyomu.jp/search?q=${encodeURIComponent(word)}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu search error: ${res.status}`)
    const apollo = parseApolloState(await res.text())
    const results: Record<string, unknown>[] = []
    for (const [key, val] of Object.entries(apollo)) {
      if (!key.startsWith('Work:')) continue
      const work = val as any
      results.push({
        id: key.replace('Work:', ''),
        title: work.title as string,
        page: (work.publicEpisodeCount ?? 0) as number,
      })
    }
    return results
  },

  async fetchWork(id: string) {
    const res = await fetch(`https://kakuyomu.jp/works/${id}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu work error: ${res.status}`)
    return parseApolloState(await res.text())
  },

  async fetchToc(id: string) {
    const apollo = await kakuyomu.fetchWork(id)
    const work = extractWork(apollo, id)
    const episodes = extractEpisodes(apollo, id)
    return {
      title: work.title,
      episodes: episodes.map((ep) => ({ num: ep.num as number, title: ep.title as string })),
    }
  },

  async fetchDetail(id: string) {
    const apollo = await kakuyomu.fetchWork(id)
    const work = extractWork(apollo, id)
    const episodes = extractEpisodes(apollo, id)
    return { title: work.title, synopsis: work.story, page: episodes.length }
  },

  async fetchDatum(id: string) {
    const apollo = await kakuyomu.fetchWork(id)
    const work = extractWork(apollo, id)
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

  async fetchPage(id: string, pageId: string | number) {
    let episodeId = String(pageId)
    const num = Number(pageId)
    // Episode IDs are 18+ digit numbers; small numbers are sequential page numbers that need resolution
    if (Number.isInteger(num) && num < 100000) {
      const apollo = await kakuyomu.fetchWork(id)
      const episodes = extractEpisodes(apollo, id)
      const ep = episodes[num - 1]
      if (!ep) throw new Error(`Episode ${pageId} not found`)
      episodeId = ep.page_id as string
    }
    const res = await fetch(`https://kakuyomu.jp/works/${id}/episodes/${episodeId}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu episode error: ${res.status}`)
    const $ = cheerio.load(await res.text())
    return $('.widget-episodeBody').html()
  },
}

export default kakuyomu
