import prisma from './prisma.js'
import modules from '../modules/index.js'

function startSyosetuSync(type: string, intervalMs: number) {
  const mod = modules[type]

  async function sync() {
    try {
      const favorites = await prisma.favorite.findMany({
        where: { type },
        select: { id: true },
      })
      if (favorites.length === 0) return

      const ids = favorites.map((f) => f.id)
      const data = await mod.fetchData(ids)

      await prisma.$transaction(
        data.map((datum) => {
          const id = datum.id as string
          const title = datum.title as string | undefined
          const page = (datum.pages as unknown[])?.length as number | undefined
          const novelupdated_at = datum.novelupdated_at as string | undefined
          return prisma.favorite.update({
            where: { type_id: { type, id } },
            data: {
              ...(title != null && { title }),
              ...(page != null && { page }),
              ...(novelupdated_at != null && { novelupdated_at }),
            },
          })
        }),
      )
      console.log(`[sync] ${type}: updated ${data.length} items`)
    } catch (e) {
      console.error(`[sync] ${type} error:`, e)
    }
  }

  sync()
  setInterval(sync, intervalMs)
}

function startKakuyomuSync() {
  const type = 'kakuyomu'
  const mod = modules[type]
  let index = 0

  async function tick() {
    try {
      const favorites = await prisma.favorite.findMany({
        where: { type },
        select: { id: true },
      })
      const count = favorites.length
      if (count === 0) {
        setTimeout(tick, 60_000)
        return
      }

      index = index % count
      const { id } = favorites[index]
      const datum = await mod.fetchDatum(id)

      const title = datum.title as string | undefined
      const page = (datum.pages as unknown[])?.length as number | undefined
      const novelupdated_at = datum.novelupdated_at as string | undefined
      await prisma.favorite.update({
        where: { type_id: { type, id } },
        data: {
          ...(title != null && { title }),
          ...(page != null && { page }),
          ...(novelupdated_at != null && { novelupdated_at }),
        },
      })
      console.log(`[sync] kakuyomu: updated ${id} (${index + 1}/${count})`)

      index++
      const interval = Math.floor(3_600_000 / count)
      setTimeout(tick, interval)
    } catch (e) {
      console.error(`[sync] kakuyomu error:`, e)
      setTimeout(tick, 60_000)
    }
  }

  tick()
}

export function startSync() {
  console.log('[sync] starting background sync')
  startSyosetuSync('narou', 10 * 60 * 1000)
  startSyosetuSync('nocturne', 10 * 60 * 1000)
  startKakuyomuSync()
}
