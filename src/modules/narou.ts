import { createFetchApi, buildPages, parsePage } from './syosetu.js'

const type = 'narou'
const fetchApi = createFetchApi('https://api.syosetu.com/novelapi/api/')

function toDatum(datum: Record<string, unknown>) {
  const { page: pageCount, ...rest } = datum
  return { ...rest, type, pages: buildPages(type, datum.id as string, pageCount as number) }
}

const narou = {
  fetchApi,

  fetchRanking(genre: string | number, limit: number, order: string = 'dailypoint') {
    return fetchApi({ of: ['t', 'w', 'n', 'ga', 'nt'], lim: limit, order, genre })
  },

  async fetchRankingList(limit: number = 100, period: string = 'daily') {
    const genres = [
      ['異世界 [恋愛]', 101], ['現実世界 [恋愛]', 102],
      ['ハイファンタジー', 201], ['ローファンタジー', 202],
      ['アクション', 306],
    ] as const
    const orderMap: Record<string, string> = {
      daily: 'dailypoint', weekly: 'weeklypoint', monthly: 'monthlypoint',
      quarter: 'quarterpoint', yearly: 'yearlypoint',
    }
    const order = orderMap[period] ?? 'dailypoint'
    const results = await Promise.all(genres.map(([, g]) => narou.fetchRanking(g, limit, order)))
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

  async fetchDetail(id: string) {
    const data = await fetchApi({ of: 't-s-ga', ncode: id })
    if (!data[0]) throw new Error('Novel not found')
    return { title: data[0].title as string, synopsis: (data[0].story as string) ?? '', page: (data[0].page as number) ?? 0 }
  },

  async fetchSearch(word: string) {
    return fetchApi({ of: ['t', 'w', 'n', 'ga', 'nt'], word, lim: 20, order: 'hyoka' })
  },

  async fetchPage(ncode: string, page: string | number) {
    let res = await fetch(`https://ncode.syosetu.com/${ncode}/${page}/`)
    if (res.status === 404) {
      res = await fetch(`https://ncode.syosetu.com/${ncode}/`)
    }
    if (!res.ok) throw new Error(`narou page error: ${res.status}`)
    return parsePage(await res.text(), '.p-novel__text')
  },
}

export default narou
