import { createFetchApi, buildPages, parsePage } from './syosetu.js'

const type = 'nocturne'
const fetchApi = createFetchApi('https://api.syosetu.com/novel18api/api/')

function toDatum(datum: Record<string, unknown>) {
  const { page: pageCount, ...rest } = datum
  return { ...rest, type, pages: buildPages(type, datum.id as string, pageCount as number) }
}

const nocturne = {
  fetchApi,

  fetchRanking(genre: string | number, limit: number, order: string = 'dailypoint') {
    return fetchApi({ of: ['t', 'w', 'n', 'ga'], lim: limit, order, nocgenre: genre })
  },

  async fetchRankingList(limit: number = 100, period: string = 'daily') {
    const orderMap: Record<string, string> = {
      daily: 'dailypoint', weekly: 'weeklypoint', monthly: 'monthlypoint',
      quarter: 'quarterpoint', yearly: 'yearlypoint',
    }
    const order = orderMap[period] ?? 'dailypoint'
    return {
      'ノクターン': await nocturne.fetchRanking(1, limit, order),
    }
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
    const res = await fetch(`https://novel18.syosetu.com/${ncode}/${page}/`, {
      headers: { Cookie: 'over18=yes' },
    })
    if (!res.ok) throw new Error(`nocturne page error: ${res.status}`)
    return parsePage(await res.text(), '.p-novel__text')
  },
}

export default nocturne
