import unittest
import std/[json, options, strutils]
import ../src/modules/syosetu

suite "mapItem":
  test "renames ncode to id and lowercases":
    let input = %*{"ncode": "N1234AB", "title": "Test"}
    let result = mapItem(input)
    check result["id"].getStr == "n1234ab"
    check result["title"].getStr == "Test"
    check not result.hasKey("ncode")

  test "renames general_all_no to page":
    let input = %*{"general_all_no": 42}
    let result = mapItem(input)
    check result["page"].getInt == 42
    check not result.hasKey("general_all_no")

  test "strips whitespace from title":
    let input = %*{"title": "  spaced title  "}
    let result = mapItem(input)
    check result["title"].getStr == "spaced title"

  test "passes through other fields unchanged":
    let input = %*{"story": "synopsis", "writer": "author"}
    let result = mapItem(input)
    check result["story"].getStr == "synopsis"
    check result["writer"].getStr == "author"

  test "maps a complete API response item":
    let input = %*{
      "ncode": "N9999ZZ",
      "title": "  異世界転生  ",
      "general_all_no": 42,
      "novelupdated_at": "2026-02-22",
    }
    let mapped = mapItem(input)
    check mapped["id"].getStr == "n9999zz"
    check mapped["title"].getStr == "異世界転生"
    check mapped["page"].getInt == 42
    check mapped["novelupdated_at"].getStr == "2026-02-22"
    check not mapped.hasKey("ncode")
    check not mapped.hasKey("general_all_no")

  test "handles null ncode without crashing":
    let input = %*{"ncode": newJNull()}
    let mapped = mapItem(input)
    check mapped["id"].kind == JNull

  test "handles null title without crashing":
    let input = %*{"title": newJNull()}
    let mapped = mapItem(input)
    check mapped["title"].kind == JNull

  test "handles numeric ncode without crashing":
    let input = %*{"ncode": 12345}
    let mapped = mapItem(input)
    check mapped.hasKey("id")

  test "handles numeric title without crashing":
    let input = %*{"title": 0}
    let mapped = mapItem(input)
    check mapped.hasKey("title")

  test "returns non-object input as-is":
    let input = %42
    let result = mapItem(input)
    check result == input

suite "processApiResponse":
  test "skips first element (metadata)":
    let input = @[%*{"allcount": 5}, %*{"ncode": "N0001A", "title": "Novel 1"}]
    let result = processApiResponse(input)
    check result.len == 1
    check result[0]["id"].getStr == "n0001a"

  test "filters out non-object elements":
    let input = @[%*{"allcount": 1}, %42, %"string", %*{"ncode": "N0002B"}]
    let result = processApiResponse(input)
    check result.len == 1
    check result[0]["id"].getStr == "n0002b"

  test "returns empty for metadata-only response":
    let input = @[%*{"allcount": 0}]
    let result = processApiResponse(input)
    check result.len == 0

  test "handles empty input":
    let input: seq[JsonNode] = @[]
    let result = processApiResponse(input)
    check result.len == 0

suite "API field constants":
  test "OfRanking uses hyphen separators":
    check '-' in OfRanking
    check ',' notin OfRanking

  test "OfDatum uses hyphen separators":
    check '-' in OfDatum
    check ',' notin OfDatum

  test "OfDetail uses hyphen separators":
    check '-' in OfDetail
    check ',' notin OfDetail

suite "buildPages":
  test "generates correct page list":
    let pages = buildPages("narou", "n1234ab", 3)
    check pages.len == 3
    check pages[0]["type"].getStr == "narou"
    check pages[0]["id"].getStr == "n1234ab"
    check pages[0]["num"].getInt == 1
    check pages[0]["page_id"].getStr == "1"
    check pages[2]["num"].getInt == 3

  test "returns empty array for zero count":
    let pages = buildPages("narou", "n1234ab", 0)
    check pages.len == 0

suite "parsePage":
  test "extracts content matching selector":
    let html = """<div class="p-novel__text"><p>第一段落</p><p>第二段落</p></div>"""
    let got = parsePage(html, ".p-novel__text")
    check got.isSome
    check "<p>第一段落</p>" in got.get
    check "<p>第二段落</p>" in got.get

  test "joins multiple matches with <hr>":
    let html = """<div class="part"><p>前編</p></div><div class="part"><p>後編</p></div>"""
    let got = parsePage(html, ".part")
    check got.isSome
    check "<hr>" in got.get

  test "returns none when selector matches nothing":
    let html = "<div>content</div>"
    let got = parsePage(html, ".nonexistent")
    check got.isNone

  test "returns none for empty matched content":
    let html = """<div class="target"></div>"""
    let got = parsePage(html, ".target")
    check got.isNone

  test "returns none for whitespace-only content":
    let html = """<div class="p-novel__text">   </div>"""
    let got = parsePage(html, ".p-novel__text")
    check got.isNone

suite "parseToc":
  test "extracts title and episodes from TOC page":
    let html = """
      <h1 class="p-novel__title">テスト小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話 始まり</a></div>
      <div class="p-eplist__sublist"><a href="/n1234ab/2/">第2話 展開</a></div>
    """
    let got = parseToc(html)
    check got.title == "テスト小説"
    check got.episodes.len == 2
    check got.episodes[0].num == 1
    check got.episodes[0].title == "第1話 始まり"
    check got.episodes[1].num == 2
    check got.episodes[1].title == "第2話 展開"
    check got.lastPage == 1

  test "falls back to <title> when .p-novel__title is missing":
    let html = """
      <html><head><title>フォールバックタイトル</title></head>
      <body>
        <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
      </body></html>
    """
    let got = parseToc(html)
    check got.title == "フォールバックタイトル"
    check got.episodes.len == 1

  test "returns empty episodes when no .p-eplist__sublist found":
    let html = """<h1 class="p-novel__title">短編小説</h1><div>本文</div>"""
    let got = parseToc(html)
    check got.title == "短編小説"
    check got.episodes.len == 0
    check got.lastPage == 1

  test "trims whitespace from title and episode titles":
    let html = """
      <h1 class="p-novel__title">  スペース付き  </h1>
      <div class="p-eplist__sublist"><a>  第1話  </a></div>
    """
    let got = parseToc(html)
    check got.title == "スペース付き"
    check got.episodes[0].title == "第1話"

  test "extracts lastPage from pagination link":
    let html = """
      <h1 class="p-novel__title">長編小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
      <a href="/n1234ab/?p=5">最後へ</a>
    """
    let got = parseToc(html)
    check got.lastPage == 5

  test "returns lastPage 1 when no pagination":
    let html = """
      <h1 class="p-novel__title">短め小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
    """
    let got = parseToc(html)
    check got.lastPage == 1
