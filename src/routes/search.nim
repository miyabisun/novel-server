import std/[asyncdispatch, json, strutils, options, httpcore]
import ../error, ../state, ../cache, ../modules/module_type

const SearchTtl* = 60 * 60  # 1 hour

proc handleSearch*(state: AppState, typeStr: string, q: string): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    let query = q.strip
    if query.len == 0:
      raise newAppError(BadRequest, "Missing query parameter: q")
    let key = "novel:" & typeStr & ":search:" & query
    let cached = state.cache.get(key)
    if cached.isSome:
      return (Http200, $cached.get)
    let results = await module.fetchSearch(state.http, query)
    state.cache.put(key, results, SearchTtl)
    return (Http200, $results)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to search"}))
