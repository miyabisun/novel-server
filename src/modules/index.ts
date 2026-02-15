import kakuyomu from './kakuyomu.js'
import narou from './narou.js'
import nocturne from './nocturne.js'

interface NovelModule {
  fetchRankingList(limit?: number): Promise<Record<string, Record<string, unknown>[]>>
  fetchPage(id: string, pageId: string | number): Promise<string | null>
}

const modules: Record<string, NovelModule> = {
  kakuyomu,
  narou,
  nocturne,
}

export default modules
