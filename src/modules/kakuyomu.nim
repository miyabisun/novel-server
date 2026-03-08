import std/[asyncdispatch, httpclient, json, strutils, uri, options, algorithm, xmltree, streams, htmlparser, times]
import nimquery
import ../error

const KakuyomuType = "kakuyomu"

type
  WorkInfo* = object
    title*: string
    story*: string
    novelupdatedAt*: Option[string]

  EpisodeInfo* = object
    num*: int
    id*: string
    title*: string

proc innerText(node: XmlNode): string =
  case node.kind
  of xnText: return node.text
  of xnElement:
    for child in node:
      result.add innerText(child)
  else: discard

proc innerHtml(node: XmlNode): string =
  for child in node:
    case child.kind
    of xnText:
      result.add child.text
    of xnElement:
      result.add $child
    else:
      discard

proc newKakuyomuClient(): AsyncHttpClient =
  newAsyncHttpClient(
    userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
  )

proc parseApolloState*(html: string): JsonNode =
  let doc = parseHtml(newStringStream(html))
  # Use attribute selector: nimquery cannot parse IDs starting with underscore
  let nodes = doc.querySelectorAll("script[id='__NEXT_DATA__']")
  if nodes.len == 0:
    raise newAppError(Upstream, "Failed to parse kakuyomu work page")
  let raw = innerText(nodes[0])
  let json = parseJson(raw)
  let apollo = json{"props", "pageProps", "__APOLLO_STATE__"}
  if apollo == nil or apollo.kind == JNull:
    raise newAppError(Upstream, "Apollo state not found")
  apollo

proc extractWork*(apollo: JsonNode, id: string): WorkInfo =
  let key = "Work:" & id
  let work = apollo{key}
  if work == nil or work.kind == JNull:
    raise newAppError(Upstream, "Work not found in Apollo state")

  let title = work{"title"}.getStr("")
  let story = work{"introduction"}.getStr("")
  var novelupdatedAt = none(string)
  let lastPub = work{"lastEpisodePublishedAt"}.getStr("")
  if lastPub.len > 0:
    try:
      # Parse ISO 8601 datetime and format as "YYYY-MM-dd HH:mm:ss"
      let dt = parse(lastPub, "yyyy-MM-dd'T'HH:mm:ss'.'fffzzz")
      novelupdatedAt = some(dt.format("yyyy-MM-dd HH:mm:ss"))
    except:
      try:
        let dt = parse(lastPub, "yyyy-MM-dd'T'HH:mm:sszzz")
        novelupdatedAt = some(dt.format("yyyy-MM-dd HH:mm:ss"))
      except:
        discard

  WorkInfo(title: title, story: story, novelupdatedAt: novelupdatedAt)

proc extractEpisodes*(apollo: JsonNode, id: string): seq[EpisodeInfo] =
  if apollo.kind != JObject:
    return @[]

  var tocKeys: seq[string] = @[]
  for key, _ in apollo:
    if key.startsWith("TableOfContentsChapter"):
      tocKeys.add(key)
  tocKeys.sort()

  var num = 0
  for tocKey in tocKeys:
    let chapter = apollo{tocKey}
    let episodeUnions = chapter{"episodeUnions"}
    if episodeUnions == nil or episodeUnions.kind != JArray:
      continue
    for epRef in episodeUnions:
      let refStr = epRef{"__ref"}.getStr("")
      if refStr.len == 0: continue
      let ep = apollo{refStr}
      if ep == nil or ep.kind == JNull: continue
      num += 1
      result.add(EpisodeInfo(
        num: num,
        id: ep{"id"}.getStr(""),
        title: ep{"title"}.getStr(""),
      ))

proc fetchRanking(genre: string, rankType: string): Future[seq[JsonNode]] {.async.} =
  let url = "https://kakuyomu.jp/rankings/" & genre & "/" & rankType
  let c = newKakuyomuClient()
  var html: string
  try:
    html = await c.getContent(url)
  finally:
    c.close()

  let doc = parseHtml(newStringStream(html))
  let workNodes = doc.querySelectorAll(".widget-work")

  result = @[]
  for elem in workNodes:
    let titleNodes = elem.querySelectorAll(".bookWalker-work-title")
    var id = ""
    var title = ""
    if titleNodes.len > 0:
      let href = titleNodes[0].attr("href")
      let parts = href.split('/')
      if parts.len > 0:
        id = parts[^1]
      title = innerText(titleNodes[0])

    let epCountNodes = elem.querySelectorAll(".widget-workCard-episodeCount")
    var page = 0
    if epCountNodes.len > 0:
      let pageText = innerText(epCountNodes[0]).replace("話", "").strip
      try: page = parseInt(pageText)
      except: discard

    result.add(%*{
      "id": id,
      "title": title,
      "page": page,
    })

proc fetchRankingList*(period: string): Future[JsonNode] {.async.} =
  if period == "quarter":
    raise newAppError(BadRequest, "kakuyomu does not support quarter ranking")
  let data = await fetchRanking("all", period)
  result = newJObject()
  result["総合"] = %data

proc fetchSearch*(word: string): Future[JsonNode] {.async.} =
  let url = "https://kakuyomu.jp/search?q=" & encodeUrl(word)
  let c = newKakuyomuClient()
  var html: string
  try:
    html = await c.getContent(url)
  finally:
    c.close()

  let apollo = parseApolloState(html)

  var results = newJArray()
  for key, val in apollo:
    if not key.startsWith("Work:"): continue
    let id = key[5 .. ^1]  # strip "Work:" prefix
    results.add(%*{
      "id": id,
      "title": val{"title"}.getStr(""),
      "page": val{"publicEpisodeCount"}.getInt(0),
    })
  return results

proc fetchWork(id: string): Future[JsonNode] {.async.} =
  let url = "https://kakuyomu.jp/works/" & id
  let c = newKakuyomuClient()
  var html: string
  try:
    html = await c.getContent(url)
  finally:
    c.close()
  return parseApolloState(html)

proc fetchToc*(id: string): Future[JsonNode] {.async.} =
  let apollo = await fetchWork(id)
  let work = extractWork(apollo, id)
  let episodes = extractEpisodes(apollo, id)
  var eps = newJArray()
  for e in episodes:
    eps.add(%*{"num": e.num, "title": e.title})
  return %*{
    "title": work.title,
    "episodes": eps,
  }

proc fetchDetail*(id: string): Future[JsonNode] {.async.} =
  let apollo = await fetchWork(id)
  let work = extractWork(apollo, id)
  let episodes = extractEpisodes(apollo, id)
  return %*{
    "title": work.title,
    "synopsis": work.story,
    "page": episodes.len,
  }

proc fetchDatum*(id: string): Future[JsonNode] {.async.} =
  let apollo = await fetchWork(id)
  let work = extractWork(apollo, id)
  let episodes = extractEpisodes(apollo, id)
  var pages = newJArray()
  for e in episodes:
    pages.add(%*{
      "type": KakuyomuType,
      "id": id,
      "num": e.num,
      "page_id": e.id,
      "title": e.title,
    })
  result = %*{
    "type": KakuyomuType,
    "id": id,
    "title": work.title,
    "story": work.story,
    "pages": pages,
  }
  if work.novelupdatedAt.isSome:
    result["novelupdated_at"] = %work.novelupdatedAt.get

proc fetchData*(ids: seq[string]): Future[seq[JsonNode]] {.async.} =
  result = @[]
  for id in ids:
    result.add(await fetchDatum(id))
    await sleepAsync(500)

proc parseEpisodePage*(html: string): Option[string] =
  let doc = parseHtml(newStringStream(html))
  let nodes = doc.querySelectorAll(".widget-episodeBody")
  if nodes.len > 0:
    let content = innerHtml(nodes[0])
    if content.len > 0:
      return some(content)
  return none(string)

proc fetchPage*(id: string, pageId: string): Future[Option[string]] {.async.} =
  var episodeId = pageId

  # Small numbers are sequential page numbers that need resolution
  try:
    let num = parseInt(pageId)
    if num < 100_000:
      let apollo = await fetchWork(id)
      let episodes = extractEpisodes(apollo, id)
      let idx = num - 1
      if idx < 0 or idx >= episodes.len:
        raise newAppError(Upstream, "Episode " & pageId & " not found")
      episodeId = episodes[idx].id
  except AppError:
    raise
  except ValueError:
    discard  # Not a number, use as-is

  let url = "https://kakuyomu.jp/works/" & id & "/episodes/" & episodeId
  let c = newKakuyomuClient()
  var html: string
  try:
    html = await c.getContent(url)
  finally:
    c.close()

  return parseEpisodePage(html)
