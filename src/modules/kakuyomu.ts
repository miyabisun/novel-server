import * as cheerio from 'cheerio'
import { format, parseISO } from 'date-fns'

const UA = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
const headers = { 'User-Agent': UA }

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

const kakuyomu = {
  async fetchRanking(genre: string, type: string) {
    const result: Record<string, unknown>[] = []
    const res = await fetch(`https://kakuyomu.jp/rankings/${genre}/${type}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu ranking error: ${res.status}`)
    const $ = cheerio.load(await res.text())
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

  async fetchRankingList() {
    return {
      '総合': await kakuyomu.fetchRanking('all', 'weekly'),
    }
  },

  async fetchDatum(id: string) {
    const res = await fetch(`https://kakuyomu.jp/works/${id}`, { headers })
    if (!res.ok) throw new Error(`kakuyomu work error: ${res.status}`)
    const html = await res.text()
    const type = 'kakuyomu'

    // Work page is CSR — extract data from __NEXT_DATA__ Apollo state
    const match = html.match(/<script id="__NEXT_DATA__"[^>]*>(.*?)<\/script>/)
    if (!match) throw new Error('Failed to parse kakuyomu work page')

    const nextData = JSON.parse(match[1])
    const apollo: Record<string, any> = nextData.props.pageProps.__APOLLO_STATE__

    // Find Work entry
    const workKey = Object.keys(apollo).find((k) => k.startsWith('Work:'))
    if (!workKey) throw new Error('Work not found in Apollo state')
    const work = apollo[workKey]

    const title = work.title as string
    const novelupdated_at = work.lastEpisodePublishedAt
      ? format(parseISO(work.lastEpisodePublishedAt), 'yyyy-MM-dd HH:mm:ss')
      : undefined

    // Collect episodes from TableOfContentsChapter entries
    const pages: Record<string, unknown>[] = []
    const tocKeys = Object.keys(apollo).filter((k) => k.startsWith('TableOfContentsChapter'))
    let num = 0
    for (const tocKey of tocKeys) {
      const toc = apollo[tocKey]
      for (const epRef of toc.episodeUnions || []) {
        const ref = epRef.__ref as string
        if (!ref || !apollo[ref]) continue
        const ep = apollo[ref]
        num++
        pages.push({
          type,
          id,
          num,
          page_id: ep.id as string,
          title: ep.title as string,
        })
      }
    }

    return { type, id, title, novelupdated_at, pages }
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
