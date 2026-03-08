import std/[asyncdispatch, httpclient, json, strutils, uri, options, xmltree, streams, htmlparser]
import nimquery
import ../error

type
  SyosetuSite* = object
    apiUrl*: string
    baseUrl*: string
    typeStr*: string
    genreParam*: string
    rankingGenres*: seq[(string, int)]
    over18*: bool

  TocEpisode* = object
    num*: int
    title*: string

  TocResult* = object
    title*: string
    episodes*: seq[TocEpisode]
    lastPage*: int

const
  OfRanking* = "t-w-n-ga-nt"
  OfDatum* = "n-t-ga-s-nu"
  OfDetail* = "t-s-ga"

let narou* = SyosetuSite(
  apiUrl: "https://api.syosetu.com/novelapi/api/",
  baseUrl: "https://ncode.syosetu.com",
  typeStr: "narou",
  genreParam: "genre",
  rankingGenres: @[
    ("異世界 [恋愛]", 101),
    ("現実世界 [恋愛]", 102),
    ("ハイファンタジー", 201),
    ("ローファンタジー", 202),
    ("アクション", 306),
  ],
  over18: false,
)

let nocturne* = SyosetuSite(
  apiUrl: "https://api.syosetu.com/novel18api/api/",
  baseUrl: "https://novel18.syosetu.com",
  typeStr: "nocturne",
  genreParam: "nocgenre",
  rankingGenres: @[("ノクターン", 1)],
  over18: true,
)

proc mapItem*(obj: JsonNode): JsonNode =
  if obj.kind != JObject:
    return obj
  result = newJObject()
  for key, val in obj:
    case key
    of "ncode":
      let id = if val.kind == JString: %val.getStr.toLowerAscii else: val
      result["id"] = id
    of "title":
      let title = if val.kind == JString: %val.getStr.strip else: val
      result["title"] = title
    of "general_all_no":
      result["page"] = val
    else:
      result[key] = val

proc buildPages*(typeStr: string, id: string, count: int): JsonNode =
  result = newJArray()
  for i in 1..count:
    result.add(%*{
      "type": typeStr,
      "id": id,
      "num": i,
      "page_id": $i,
    })

proc processApiResponse*(json: seq[JsonNode]): seq[JsonNode] =
  result = @[]
  for i, v in json:
    if i == 0: continue  # skip metadata
    if v.kind == JObject:
      result.add(mapItem(v))

proc newSiteClient(site: SyosetuSite): AsyncHttpClient =
  result = newAsyncHttpClient(
    userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
  )
  if site.over18:
    result.headers = newHttpHeaders({"Cookie": "over18=yes"})

proc fetchApi*(client: AsyncHttpClient, apiUrl: string,
               params: seq[(string, string)],
               headers: HttpHeaders = nil): Future[seq[JsonNode]] {.async.} =
  var allParams = @[("out", "json")]
  allParams.add(params)

  var queryParts: seq[string] = @[]
  for (k, v) in allParams:
    queryParts.add(encodeUrl(k) & "=" & encodeUrl(v))
  let url = apiUrl & "?" & queryParts.join("&")

  let httpClient = newAsyncHttpClient(
    userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
  )
  if headers != nil:
    httpClient.headers = headers

  let res = await httpClient.getContent(url)
  httpClient.close()

  let json = parseJson(res)
  if json.kind != JArray:
    raise newAppError(Upstream, "API returned non-array response")
  var items: seq[JsonNode] = @[]
  for item in json:
    items.add(item)
  return processApiResponse(items)

proc siteApi(site: SyosetuSite, params: seq[(string, string)]): Future[seq[JsonNode]] {.async.} =
  let headers = if site.over18:
    newHttpHeaders({"Cookie": "over18=yes"})
  else:
    nil
  return await fetchApi(
    newAsyncHttpClient(),
    site.apiUrl,
    params,
    headers,
  )

proc toDatum(site: SyosetuSite, datum: JsonNode): JsonNode =
  let id = datum{"id"}.getStr("")
  let pageCount = datum{"page"}.getInt(0)
  let pages = buildPages(site.typeStr, id, pageCount)
  result = datum.copy()
  if result.hasKey("page"):
    result.delete("page")
  result["type"] = %site.typeStr
  result["pages"] = pages

proc tocToJson(title: string, episodes: seq[TocEpisode]): JsonNode =
  var eps = newJArray()
  for e in episodes:
    eps.add(%*{"num": e.num, "title": e.title})
  %*{"title": title, "episodes": eps}

proc fetchRanking(site: SyosetuSite, genre: int, limit: int, order: string): Future[seq[JsonNode]] {.async.} =
  return await siteApi(site, @[
    ("of", OfRanking),
    ("lim", $limit),
    ("order", order),
    (site.genreParam, $genre),
  ])

proc fetchRankingList*(site: SyosetuSite, client: AsyncHttpClient, limit: int, period: string): Future[JsonNode] {.async.} =
  let order = case period
    of "daily": "dailypoint"
    of "weekly": "weeklypoint"
    of "monthly": "monthlypoint"
    of "quarter": "quarterpoint"
    of "yearly": "yearlypoint"
    else: "dailypoint"

  var futures: seq[Future[seq[JsonNode]]] = @[]
  for (_, genreId) in site.rankingGenres:
    futures.add(fetchRanking(site, genreId, limit, order))

  result = newJObject()
  for i, fut in futures:
    let data = await fut
    result[site.rankingGenres[i][0]] = %data

proc fetchDatum*(site: SyosetuSite, client: AsyncHttpClient, id: string): Future[JsonNode] {.async.} =
  let data = await siteApi(site, @[
    ("of", OfDatum),
    ("ncode", id),
  ])
  if data.len == 0:
    raise newAppError(Upstream, "Novel not found")
  return toDatum(site, data[0])

proc fetchData*(site: SyosetuSite, client: AsyncHttpClient, ids: seq[string]): Future[seq[JsonNode]] {.async.} =
  let ncodeStr = ids.join("-")
  let data = await siteApi(site, @[
    ("of", OfDatum),
    ("ncode", ncodeStr),
  ])
  result = @[]
  for d in data:
    result.add(toDatum(site, d))

proc fetchDetail*(site: SyosetuSite, client: AsyncHttpClient, id: string): Future[JsonNode] {.async.} =
  let data = await siteApi(site, @[
    ("of", OfDetail),
    ("ncode", id),
  ])
  if data.len == 0:
    raise newAppError(Upstream, "Novel not found")
  let item = data[0]
  return %*{
    "title": item{"title"}.getStr(""),
    "synopsis": item{"story"}.getStr(""),
    "page": item{"page"}.getInt(0),
  }

proc fetchSearch*(site: SyosetuSite, client: AsyncHttpClient, word: string): Future[JsonNode] {.async.} =
  let data = await siteApi(site, @[
    ("of", OfRanking),
    ("word", word),
    ("lim", "20"),
    ("order", "hyoka"),
  ])
  return %data

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

proc parseToc*(html: string): TocResult =
  let doc = parseHtml(newStringStream(html))

  # Title: .p-novel__title
  var title = ""
  let titleNodes = doc.querySelectorAll(".p-novel__title")
  if titleNodes.len > 0:
    title = innerText(titleNodes[0]).strip
  if title.len == 0:
    let titleFallback = doc.querySelectorAll("title")
    if titleFallback.len > 0:
      title = innerText(titleFallback[0]).strip

  # Episodes: .p-eplist__sublist
  var episodes: seq[TocEpisode] = @[]
  var num = 0
  let epNodes = doc.querySelectorAll(".p-eplist__sublist")
  for el in epNodes:
    num += 1
    var epTitle = ""
    let aNodes = el.querySelectorAll("a")
    if aNodes.len > 0:
      epTitle = innerText(aNodes[0]).strip
    episodes.add(TocEpisode(num: num, title: epTitle))

  # Last page: find <a> with text "最後へ"
  var lastPage = 1
  let allLinks = doc.querySelectorAll("a")
  for el in allLinks:
    let text = innerText(el).strip
    if text == "最後へ":
      let href = el.attr("href")
      let pIdx = href.find("p=")
      if pIdx >= 0:
        let rest = href[pIdx + 2 .. ^1]
        var numStr = ""
        for c in rest:
          if c in '0'..'9': numStr.add(c)
          else: break
        if numStr.len > 0:
          try: lastPage = parseInt(numStr)
          except: discard

  TocResult(title: title, episodes: episodes, lastPage: lastPage)

proc parsePage*(html: string, selector: string): Option[string] =
  let doc = parseHtml(newStringStream(html))
  let nodes = doc.querySelectorAll(selector)
  var parts: seq[string] = @[]
  for el in nodes:
    let h = innerHtml(el)
    if h.strip.len > 0:
      parts.add(h)
  if parts.len == 0:
    none(string)
  else:
    some(parts.join("<hr>"))

proc fetchToc*(site: SyosetuSite, client: AsyncHttpClient, ncode: string): Future[JsonNode] {.async.} =
  let baseUrl = site.baseUrl & "/" & ncode & "/"

  let siteClient = newSiteClient(site)
  let firstHtml = await siteClient.getContent(baseUrl)
  siteClient.close()

  let first = parseToc(firstHtml)

  if first.lastPage <= 1:
    return tocToJson(first.title, first.episodes)

  var allEpisodes = first.episodes
  var pageFutures: seq[Future[string]] = @[]
  for page in 2..first.lastPage:
    let url = baseUrl & "?p=" & $page
    let c = newSiteClient(site)
    pageFutures.add((proc(cl: AsyncHttpClient, u: string): Future[string] {.async.} =
      let res = await cl.getContent(u)
      cl.close()
      return res
    )(c, url))

  for fut in pageFutures:
    let pageHtml = await fut
    let toc = parseToc(pageHtml)
    for ep in toc.episodes:
      allEpisodes.add(TocEpisode(num: allEpisodes.len + 1, title: ep.title))

  # Renumber episodes
  var eps: seq[TocEpisode] = @[]
  for i, ep in allEpisodes:
    eps.add(TocEpisode(num: i + 1, title: ep.title))

  tocToJson(first.title, eps)

proc fetchPage*(site: SyosetuSite, client: AsyncHttpClient, ncode: string, page: string): Future[Option[string]] {.async.} =
  let url = site.baseUrl & "/" & ncode & "/" & page & "/"
  let siteClient = newSiteClient(site)

  var html: string
  try:
    html = await siteClient.getContent(url)
  except HttpRequestError:
    # Try fallback URL (single page novel)
    let fallbackUrl = site.baseUrl & "/" & ncode & "/"
    try:
      html = await siteClient.getContent(fallbackUrl)
    except HttpRequestError as e:
      siteClient.close()
      raise newAppError(Upstream, site.typeStr & " page error: " & e.msg)
  finally:
    siteClient.close()

  return parsePage(html, ".p-novel__text")
