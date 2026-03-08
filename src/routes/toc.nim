import std/[asyncdispatch, json, logging, httpcore]
import ../error, ../state, ../modules/module_type

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

proc handleToc*(state: AppState, typeStr: string, id: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let label = "fetchToc " & typeStr & "/" & id
    let toc = await withRetryJson(label, proc(): Future[JsonNode] {.async.} =
      return await module.fetchToc(id)
    )
    return (Http200, $toc)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to fetch toc"}))
