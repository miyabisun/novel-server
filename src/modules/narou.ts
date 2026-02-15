import * as cheerio from 'cheerio'

const type = 'narou'
const apiUrl = 'https://api.syosetu.com/novelapi/api/'

function fetchApi(q: Record<string, unknown>) {
  const params: Record<string, string> = { out: 'json' }
  for (const [key, val] of Object.entries(q)) {
    params[key] = Array.isArray(val) ? val.join('-') : String(val)
  }
  const query = new URLSearchParams(params)

  return fetch(`${apiUrl}?${query}`)
    .then((res) => {
      if (!res.ok) throw new Error(`narou API error: ${res.status}`)
      return res.json()
    })
    .then((json: any[]) =>
      json.slice(1).map((obj: Record<string, unknown>) => {
        const acc: Record<string, unknown> = {}
        for (const [key, val] of Object.entries(obj)) {
          switch (key) {
            case 'ncode':
              acc['id'] = (val as string).toLowerCase()
              break
            case 'title':
              acc[key] = (val as string).trim()
              break
            case 'general_all_no':
              acc['page'] = val
              break
            default:
              acc[key] = val
          }
        }
        return acc
      }),
    )
}

const narou = {
  fetchApi,

  fetchRanking(genre: string | number, limit: number) {
    return fetchApi({ of: 't-w-n-ga', lim: limit, order: 'dailypoint', genre })
  },

  async fetchRankingList(limit: number = 100) {
    return {
      '異世界 [恋愛]': await narou.fetchRanking(101, limit),
      '現実世界 [恋愛]': await narou.fetchRanking(102, limit),
      'ハイファンタジー': await narou.fetchRanking(201, limit),
      'ローファンタジー': await narou.fetchRanking(202, limit),
      'アクション': await narou.fetchRanking(306, limit),
    }
  },

  async fetchDatum(id: string) {
    const data = await fetchApi({ of: ['n', 't', 'ga', 's', 'nu'], ncode: id })
    const datum = data[0] as Record<string, unknown>
    const pageCount = datum.page as number

    const pages = Array.from({ length: pageCount }, (_, i) => ({
      type,
      id,
      num: i + 1,
      page_id: String(i + 1),
    }))

    const { page: _, ...rest } = datum
    return { ...rest, type, pages }
  },

  async fetchData(ids: string[]) {
    const data = await fetchApi({ of: ['n', 't', 'ga', 's', 'nu'], ncode: ids })
    return data.map((datum: Record<string, unknown>) => {
      const pageCount = datum.page as number
      const itemId = datum.id as string
      const pages = Array.from({ length: pageCount }, (_, i) => ({
        type,
        id: itemId,
        num: i + 1,
        page_id: String(i + 1),
      }))
      const { page: _, ...rest } = datum
      return { ...rest, type, pages }
    })
  },

  async fetchPage(ncode: string, page: string | number) {
    const res = await fetch(`https://ncode.syosetu.com/${ncode}/${page}/`)
    if (!res.ok) throw new Error(`narou page error: ${res.status}`)
    const $ = cheerio.load(await res.text())
    return $('.p-novel__text').html()
  },
}

export default narou
