import { createFetchApi, buildPages, parsePage } from './syosetu.js'

const type = 'narou'
const fetchApi = createFetchApi('https://api.syosetu.com/novelapi/api/')

function toDatum(datum: Record<string, unknown>) {
  const { page: pageCount, ...rest } = datum
  return { ...rest, type, pages: buildPages(type, datum.id as string, pageCount as number) }
}

const narou = {
  fetchApi,

  fetchRanking(genre: string | number, limit: number) {
    return fetchApi({ of: 't-w-n-ga', lim: limit, order: 'dailypoint', genre })
  },

  async fetchRankingList(limit: number = 100) {
    const genres = [
      ['異世界 [恋愛]', 101], ['現実世界 [恋愛]', 102],
      ['ハイファンタジー', 201], ['ローファンタジー', 202],
      ['アクション', 306],
    ] as const
    const results = await Promise.all(genres.map(([, g]) => narou.fetchRanking(g, limit)))
    return Object.fromEntries(genres.map(([name], i) => [name, results[i]]))
  },

  async fetchDatum(id: string) {
    const data = await fetchApi({ of: ['n', 't', 'ga', 's', 'nu'], ncode: id })
    return toDatum(data[0])
  },

  async fetchData(ids: string[]) {
    const data = await fetchApi({ of: ['n', 't', 'ga', 's', 'nu'], ncode: ids })
    return data.map(toDatum)
  },

  async fetchPage(ncode: string, page: string | number) {
    const res = await fetch(`https://ncode.syosetu.com/${ncode}/${page}/`)
    if (!res.ok) throw new Error(`narou page error: ${res.status}`)
    return parsePage(await res.text(), '.p-novel__text')
  },
}

export default narou
