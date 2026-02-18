import { eq, and } from 'drizzle-orm'
import { db } from '../db/index.js'
import { favorites } from '../db/schema.js'
import modules from '../modules/index.js'

function startSyosetuSync(type: string, intervalMs: number) {
  const mod = modules[type]

  async function sync() {
    try {
      const rows = db.select({ id: favorites.id }).from(favorites)
        .where(eq(favorites.type, type))
        .all()
      if (rows.length === 0) return

      const ids = rows.map((f) => f.id)
      const data = await mod.fetchData(ids)

      db.transaction((tx) => {
        for (const datum of data) {
          const id = datum.id as string
          const title = datum.title as string | undefined
          const page = (datum.pages as unknown[])?.length as number | undefined
          const novelupdated_at = datum.novelupdated_at as string | undefined
          tx.update(favorites)
            .set({
              ...(title != null && { title }),
              ...(page != null && { page }),
              ...(novelupdated_at != null && { novelupdated_at }),
            })
            .where(and(eq(favorites.type, type), eq(favorites.id, id)))
            .run()
        }
      })
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
      const rows = db.select({ id: favorites.id }).from(favorites)
        .where(eq(favorites.type, type))
        .all()
      const count = rows.length
      if (count === 0) {
        setTimeout(tick, 60_000)
        return
      }

      index = index % count
      const { id } = rows[index]
      const datum = await mod.fetchDatum(id)

      const title = datum.title as string | undefined
      const page = (datum.pages as unknown[])?.length as number | undefined
      const novelupdated_at = datum.novelupdated_at as string | undefined
      db.update(favorites)
        .set({
          ...(title != null && { title }),
          ...(page != null && { page }),
          ...(novelupdated_at != null && { novelupdated_at }),
        })
        .where(and(eq(favorites.type, type), eq(favorites.id, id)))
        .run()
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
