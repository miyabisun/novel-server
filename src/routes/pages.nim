import std/[asyncdispatch, json, options, logging, httpcore]
import ../error, ../state, ../cache, ../modules/module_type, ../sanitize

const PageTtl* = 60 * 60 * 24  # 24 hours

proc withRetryOpt(label: string, body: proc(): Future[Option[string]]): Future[Option[string]] {.async.} =
  for i in 0..2:
    try:
      return await body()
    except AppError:
      raise
    except CatchableError as e:
      error label & " attempt " & $(i + 1) & " failed: " & e.msg
      if i < 2:
        await sleepAsync(500 * (i + 1))
  raise newAppError(Upstream, "Failed after 3 retries: " & label)

proc fetchAndCache(state: AppState, module: ModuleType, id: string, num: string, key: string): Future[JsonNode] {.async.} =
  let label = "fetchPage " & id & "/" & num
  let raw = await withRetryOpt(label, proc(): Future[Option[string]] {.async.} =
    return await module.fetchPage(id, num)
  )
  let html = clean(raw.get(""))
  state.cache.put(key, %html, PageTtl)
  return %*{"html": html}

proc handleGetPage*(state: AppState, typeStr: string, id: string, num: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let key = "novel:" & typeStr & ":" & id & ":page:" & num
    let cached = state.cache.get(key)
    if cached.isSome:
      return (Http200, $(%*{"html": cached.get}))
    let res = await fetchAndCache(state, module, id, num, key)
    return (Http200, $res)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to fetch page"}))

proc handlePatchPage*(state: AppState, typeStr: string, id: string, num: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let key = "novel:" & typeStr & ":" & id & ":page:" & num
    let res = await fetchAndCache(state, module, id, num, key)
    return (Http200, $res)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to fetch page"}))
