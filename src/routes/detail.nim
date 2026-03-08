import std/[asyncdispatch, json, options, logging, httpcore]
import ../error, ../state, ../cache, ../modules/module_type

const DetailTtl* = 60 * 60 * 24  # 24 hours

proc withRetryJson(label: string, body: proc(): Future[JsonNode]): Future[JsonNode] {.async.} =
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

proc handleDetail*(state: AppState, typeStr: string, id: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let key = "novel:" & typeStr & ":" & id & ":detail"
    let cached = state.cache.get(key)
    if cached.isSome:
      return (Http200, $cached.get)
    let label = "fetchDetail " & typeStr & "/" & id
    let detail = await withRetryJson(label, proc(): Future[JsonNode] {.async.} =
      return await module.fetchDetail(id)
    )
    state.cache.put(key, detail, DetailTtl)
    return (Http200, $detail)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to fetch detail"}))
