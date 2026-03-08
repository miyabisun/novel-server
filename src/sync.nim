import std/[asyncdispatch, json, logging, strutils, locks]
import db_connector/db_sqlite
import state, modules/module_type

proc getIds(state: AppState, typeStr: string): seq[string] =
  result = @[]
  acquire(state.dbLock)
  try:
    let rows = state.db.getAllRows(sql"SELECT id FROM favorites WHERE type = ?", typeStr)
    for row in rows:
      result.add(row[0])
  except DbError as e:
    error "[sync] " & typeStr & " db error: " & e.msg
  finally:
    release(state.dbLock)

proc updateFavoriteFromDatum*(state: AppState, typeStr: string, datum: JsonNode) =
  let id = datum{"id"}.getStr("")
  let title = datum{"title"}.getStr("")
  let pages = datum{"pages"}
  let pageCount = if pages != nil and pages.kind == JArray: pages.len else: -1
  let novelupdatedAt = datum{"novelupdated_at"}.getStr("")

  if title.len == 0 and pageCount < 0 and novelupdatedAt.len == 0:
    return

  acquire(state.dbLock)
  try:
    state.db.exec(sql"""
      UPDATE favorites SET
        title = COALESCE(NULLIF(?, ''), title),
        page = COALESCE(NULLIF(?, -1), page),
        novelupdated_at = COALESCE(NULLIF(?, ''), novelupdated_at)
      WHERE type = ? AND id = ?
    """, title, pageCount, novelupdatedAt, typeStr, id)
  except DbError as e:
    error "[sync] update error: " & e.msg
  finally:
    release(state.dbLock)

proc syncSyosetu(state: AppState, module: ModuleType, typeStr: string) {.async.} =
  let ids = getIds(state, typeStr)
  if ids.len == 0: return

  try:
    let data = await module.fetchData(state.http, ids)
    acquire(state.dbLock)
    try:
      for datum in data:
        let id = datum{"id"}.getStr("")
        let title = datum{"title"}.getStr("")
        let pages = datum{"pages"}
        let pageCount = if pages != nil and pages.kind == JArray: pages.len else: -1
        let novelupdatedAt = datum{"novelupdated_at"}.getStr("")

        if title.len > 0 or pageCount >= 0 or novelupdatedAt.len > 0:
          state.db.exec(sql"""
            UPDATE favorites SET
              title = COALESCE(NULLIF(?, ''), title),
              page = COALESCE(NULLIF(?, -1), page),
              novelupdated_at = COALESCE(NULLIF(?, ''), novelupdated_at)
          WHERE type = ? AND id = ?
          """, title, pageCount, novelupdatedAt, typeStr, id)
    finally:
      release(state.dbLock)
    info "[sync] " & typeStr & ": updated " & $data.len & " items"
  except CatchableError as e:
    error "[sync] " & typeStr & " error: " & e.msg

proc startSyosetuSync(state: AppState, module: ModuleType, intervalMs: int) {.async.} =
  let typeStr = module.asStr
  # Initial sync
  await syncSyosetu(state, module, typeStr)
  while true:
    await sleepAsync(intervalMs)
    await syncSyosetu(state, module, typeStr)

proc startKakuyomuSync(state: AppState) {.async.} =
  let module = ModuleType.Kakuyomu
  let typeStr = "kakuyomu"
  var index = 0

  while true:
    let ids = getIds(state, typeStr)
    let count = ids.len
    if count == 0:
      await sleepAsync(60_000)
      continue

    index = index mod count
    let id = ids[index]

    try:
      let datum = await module.fetchDatum(state.http, id)
      updateFavoriteFromDatum(state, typeStr, datum)
      info "[sync] kakuyomu: updated " & id & " (" & $(index + 1) & "/" & $count & ")"
      index += 1
      let intervalMs = 3_600_000 div count
      await sleepAsync(intervalMs)
    except CatchableError as e:
      error "[sync] kakuyomu error: " & e.msg
      await sleepAsync(60_000)

proc startSync*(state: AppState) =
  info "[sync] starting background sync"
  asyncCheck startSyosetuSync(state, ModuleType.Narou, 600_000)
  asyncCheck startSyosetuSync(state, ModuleType.Nocturne, 600_000)
  asyncCheck startKakuyomuSync(state)
