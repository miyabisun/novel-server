import kakuyomu from './kakuyomu.js'
import narou from './narou.js'
import nocturne from './nocturne.js'

interface NovelModule {
  fetchRankingList(limit?: number, period?: string): Promise<Record<string, Record<string, unknown>[]>>
  fetchPage(id: string, pageId: string | number): Promise<string | null>
  fetchDetail(id: string): Promise<{ title: string; synopsis: string }>
}

const modules: Record<string, NovelModule> = {
  kakuyomu,
  narou,
  nocturne,
}

export default modules
