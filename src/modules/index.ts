import kakuyomu from './kakuyomu.js'
import narou from './narou.js'
import nocturne from './nocturne.js'

interface NovelModule {
  fetchRankingList(limit?: number, period?: string): Promise<Record<string, Record<string, unknown>[]>>
  fetchPage(id: string, pageId: string | number): Promise<string | null>
  fetchDetail(id: string): Promise<{ title: string; synopsis: string; page: number }>
  fetchSearch(word: string): Promise<Record<string, unknown>[]>
  fetchToc(id: string): Promise<{ title: string; episodes: { num: number; title: string }[] }>
  fetchData(ids: string[]): Promise<Record<string, unknown>[]>
  fetchDatum(id: string): Promise<Record<string, unknown>>
}

const modules: Record<string, NovelModule> = {
  kakuyomu,
  narou,
  nocturne,
}

export default modules
