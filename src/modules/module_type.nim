import std/[asyncdispatch, httpclient, json, options]
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

proc fetchRankingList*(m: ModuleType, client: AsyncHttpClient, limit: int, period: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchRankingList(narou, client, limit, period)
  of Nocturne: return await syosetu.fetchRankingList(nocturne, client, limit, period)
  of Kakuyomu: return await kakuyomu.fetchRankingList(client, period)

proc fetchPage*(m: ModuleType, client: AsyncHttpClient, id: string, pageId: string): Future[Option[string]] {.async.} =
  case m
  of Narou: return await syosetu.fetchPage(narou, client, id, pageId)
  of Nocturne: return await syosetu.fetchPage(nocturne, client, id, pageId)
  of Kakuyomu: return await kakuyomu.fetchPage(client, id, pageId)

proc fetchDetail*(m: ModuleType, client: AsyncHttpClient, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchDetail(narou, client, id)
  of Nocturne: return await syosetu.fetchDetail(nocturne, client, id)
  of Kakuyomu: return await kakuyomu.fetchDetail(client, id)

proc fetchSearch*(m: ModuleType, client: AsyncHttpClient, word: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchSearch(narou, client, word)
  of Nocturne: return await syosetu.fetchSearch(nocturne, client, word)
  of Kakuyomu: return await kakuyomu.fetchSearch(client, word)

proc fetchToc*(m: ModuleType, client: AsyncHttpClient, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchToc(narou, client, id)
  of Nocturne: return await syosetu.fetchToc(nocturne, client, id)
  of Kakuyomu: return await kakuyomu.fetchToc(client, id)

proc fetchData*(m: ModuleType, client: AsyncHttpClient, ids: seq[string]): Future[seq[JsonNode]] {.async.} =
  case m
  of Narou: return await syosetu.fetchData(narou, client, ids)
  of Nocturne: return await syosetu.fetchData(nocturne, client, ids)
  of Kakuyomu: return await kakuyomu.fetchData(client, ids)

proc fetchDatum*(m: ModuleType, client: AsyncHttpClient, id: string): Future[JsonNode] {.async.} =
  case m
  of Narou: return await syosetu.fetchDatum(narou, client, id)
  of Nocturne: return await syosetu.fetchDatum(nocturne, client, id)
  of Kakuyomu: return await kakuyomu.fetchDatum(client, id)
