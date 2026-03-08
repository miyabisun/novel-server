import std/[asyncdispatch, json, options]
import ../error
import syosetu, kakuyomu

type ModuleType* = enum
  Narou, Nocturne, Kakuyomu

proc resolve*(s: string): ModuleType =
  case s
  of "narou": Narou
  of "nocturne": Nocturne
  of "kakuyomu": Kakuyomu
  else: raise newAppError(BadRequest, "Invalid type")

proc asStr*(m: ModuleType): string =
  case m
  of Narou: "narou"
  of Nocturne: "nocturne"
  of Kakuyomu: "kakuyomu"

proc fetchRankingList*(m: ModuleType, limit: int, period: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchRankingList(narou, limit, period)
  of Nocturne: return await syosetu.fetchRankingList(nocturne, limit, period)
  of Kakuyomu: return await kakuyomu.fetchRankingList(period)

proc fetchPage*(m: ModuleType, id: string, pageId: string): Future[Option[string]] {.async.} =
  case m
  of Narou: return await syosetu.fetchPage(narou, id, pageId)
  of Nocturne: return await syosetu.fetchPage(nocturne, id, pageId)
  of Kakuyomu: return await kakuyomu.fetchPage(id, pageId)

proc fetchDetail*(m: ModuleType, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchDetail(narou, id)
  of Nocturne: return await syosetu.fetchDetail(nocturne, id)
  of Kakuyomu: return await kakuyomu.fetchDetail(id)

proc fetchSearch*(m: ModuleType, word: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchSearch(narou, word)
  of Nocturne: return await syosetu.fetchSearch(nocturne, word)
  of Kakuyomu: return await kakuyomu.fetchSearch(word)

proc fetchToc*(m: ModuleType, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchToc(narou, id)
  of Nocturne: return await syosetu.fetchToc(nocturne, id)
  of Kakuyomu: return await kakuyomu.fetchToc(id)

proc fetchData*(m: ModuleType, ids: seq[string]): Future[seq[JsonNode]] {.async.} =
  case m
  of Narou: return await syosetu.fetchData(narou, ids)
  of Nocturne: return await syosetu.fetchData(nocturne, ids)
  of Kakuyomu: return await kakuyomu.fetchData(ids)

proc fetchDatum*(m: ModuleType, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchDatum(narou, id)
  of Nocturne: return await syosetu.fetchDatum(nocturne, id)
  of Kakuyomu: return await kakuyomu.fetchDatum(id)
