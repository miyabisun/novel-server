import std/[asyncdispatch, json, strutils, options, httpcore]
import ../error, ../state, ../cache, ../modules/module_type

const
  RankingTtl* = 60 * 60 * 3  # 3 hours
  ValidPeriods = ["daily", "weekly", "monthly", "quarter", "yearly"]
  QuarterUnsupported = ["kakuyomu"]

proc validatePeriod(typeStr: string, period: string) =
  var valid = false
  for p in ValidPeriods:
    if p == period:
      valid = true
      break
  if not valid:
    raise newAppError(BadRequest, "Invalid period")
  if period == "quarter":
    for t in QuarterUnsupported:
      if t == typeStr:
        raise newAppError(BadRequest, typeStr & " does not support quarter ranking")

proc handleRanking*(state: AppState, typeStr: string, period: string, useCache: bool): Future[(HttpCode, string)] {.async.} =
  try:
    let module = resolve(typeStr)
    validatePeriod(typeStr, period)
    let key = "novel:" & typeStr & ":ranking:" & period
    if useCache:
      let cached = state.cache.get(key)
      if cached.isSome:
        return (Http200, $cached.get)
    let ranking = await module.fetchRankingList(100, period)
    state.cache.put(key, ranking, RankingTtl)
    return (Http200, $ranking)
  except AppError as e:
    return (HttpCode(e.kind.statusCode), $e.toJson)
  except CatchableError:
    return (Http502, $(%*{"error": "Failed to fetch ranking"}))
