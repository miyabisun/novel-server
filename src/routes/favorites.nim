import std/[asyncdispatch, json, strutils, options, logging, locks, httpcore]
import db_connector/db_sqlite
import ../error, ../state, ../modules/module_type, ../sync

proc mapFavoriteRow(row: Row): JsonNode =
  %*{
    "type": row[0],
    "id": row[1],
    "title": row[2],
    "novelupdated_at": (if row[3].len > 0: %row[3] else: newJNull()),
    "page": parseInt(row[4]),
    "read": parseInt(row[5]),
  }

proc handleGetFavorites*(state: AppState): Future[(HttpCode, string)] {.async.} =
  try:
    var rows: seq[JsonNode] = @[]
    acquire(state.dbLock)
    try:
      let dbRows = state.db.getAllRows(sql"""
        SELECT type, id, title, COALESCE(novelupdated_at, ''), page, read
        FROM favorites ORDER BY novelupdated_at DESC NULLS LAST
      """)
      for row in dbRows:
        rows.add(mapFavoriteRow(row))
    finally:
      release(state.dbLock)
    return (Http200, $(%rows))
  except CatchableError as e:
    return (Http500, $(%*{"error": e.msg}))

proc handlePutFavorite*(state: AppState, bodyStr: string, typeStr: string, id: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let body = parseJson(bodyStr)
    let title = body{"title"}.getStr("")
    if title.len == 0:
      raise newAppError(BadRequest, "title and page are required")
    let page = body{"page"}.getInt(-1)
    if page < 0:
      raise newAppError(BadRequest, "title and page are required")
    let novelupdatedAt = body{"novelupdated_at"}.getStr("")

    var favorite: JsonNode
    acquire(state.dbLock)
    try:
      state.db.exec(sql"""
        INSERT INTO favorites (type, id, title, page, novelupdated_at, read) VALUES (?, ?, ?, ?, ?, 0)
        ON CONFLICT(type, id) DO UPDATE SET title = ?, page = ?, novelupdated_at = ?
      """, typeStr, id, title, page,
        (if novelupdatedAt.len > 0: novelupdatedAt else: ""),
        title, page,
        (if novelupdatedAt.len > 0: novelupdatedAt else: ""))
      let row = state.db.getRow(sql"""
        SELECT type, id, title, COALESCE(novelupdated_at, ''), page, read
        FROM favorites WHERE type = ? AND id = ?
      """, typeStr, id)
      favorite = mapFavoriteRow(row)
    finally:
      release(state.dbLock)

    # Fire-and-forget: fetch metadata immediately after adding
    asyncCheck (proc() {.async.} =
      try:
        let datum = await module.fetchDatum(id)
        updateFavoriteFromDatum(state, typeStr, datum)
        info "[sync] initial fetch for " & typeStr & "/" & id
      except CatchableError as e:
        error "[sync] initial fetch failed for " & typeStr & "/" & id & ": " & e.msg
    )()

    return (Http200, $favorite)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError as e:
    return (Http500, $(%*{"error": e.msg}))

proc handleDeleteFavorite*(state: AppState, typeStr: string, id: string): Future[(HttpCode, string)] {.async.} =
  try:
    discard resolve(typeStr)
    acquire(state.dbLock)
    try:
      state.db.exec(sql"DELETE FROM favorites WHERE type = ? AND id = ?", typeStr, id)
    finally:
      release(state.dbLock)
    return (Http200, $(%*{"ok": true}))
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError as e:
    return (Http500, $(%*{"error": e.msg}))

proc handlePatchProgress*(state: AppState, bodyStr: string, typeStr: string, id: string): Future[(HttpCode, string)] {.async.} =
  try:
    discard resolve(typeStr)
    let body = parseJson(bodyStr)
    let read = body{"read"}.getInt(-1)
    if read < 0:
      raise newAppError(BadRequest, "read is required")

    var resultJson: JsonNode
    acquire(state.dbLock)
    try:
      state.db.exec(sql"UPDATE favorites SET read = ? WHERE type = ? AND id = ?",
        read, typeStr, id)
      let row = state.db.getRow(sql"""
        SELECT type, id, title, COALESCE(novelupdated_at, ''), page, read
        FROM favorites WHERE type = ? AND id = ?
      """, typeStr, id)
      if row[0].len == 0:
        resultJson = %*{"ok": true}
      else:
        resultJson = mapFavoriteRow(row)
    finally:
      release(state.dbLock)
    return (Http200, $resultJson)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError as e:
    return (Http500, $(%*{"error": e.msg}))
