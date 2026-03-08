import unittest
import std/[json, options, strutils]
import ../src/modules/kakuyomu

const WorkId = "1177354054882725960"
const WorkHtml = staticRead("fixtures/kakuyomu_work.html")
const EpisodeHtml = staticRead("fixtures/kakuyomu_episode.html")

suite "parseApolloState":
  test "parses __NEXT_DATA__ from work page":
    let apollo = parseApolloState(WorkHtml)
    check apollo.kind == JObject
    check apollo.len > 0

  test "contains Work entry for target id":
    let apollo = parseApolloState(WorkHtml)
    let work = apollo{"Work:" & WorkId}
    check work != nil
    check work.kind == JObject

  test "contains Episode entries":
    let apollo = parseApolloState(WorkHtml)
    var episodeCount = 0
    for key, _ in apollo:
      if key.startsWith("Episode:"):
        episodeCount += 1
    check episodeCount > 0

  test "raises on HTML without __NEXT_DATA__":
    expect(CatchableError):
      discard parseApolloState("<html><body>no data</body></html>")

suite "extractWork":
  let apollo = parseApolloState(WorkHtml)

  test "extracts title":
    let work = extractWork(apollo, WorkId)
    check work.title.len > 0
    check "チート能力" in work.title

  test "extracts story (introduction)":
    let work = extractWork(apollo, WorkId)
    check work.story.len > 0
    check "天上優夜" in work.story

  test "extracts lastEpisodePublishedAt as formatted date":
    let work = extractWork(apollo, WorkId)
    if work.novelupdatedAt.isSome:
      let dt = work.novelupdatedAt.get
      check dt.contains("-")
      check dt.contains(":")

  test "raises for non-existent work id":
    expect(CatchableError):
      discard extractWork(apollo, "0000000000000000000")

suite "extractEpisodes":
  let apollo = parseApolloState(WorkHtml)

  test "returns non-empty episode list":
    let episodes = extractEpisodes(apollo, WorkId)
    check episodes.len > 0

  test "episodes have sequential numbering starting at 1":
    let episodes = extractEpisodes(apollo, WorkId)
    check episodes[0].num == 1
    for i, ep in episodes:
      check ep.num == i + 1

  test "episodes have non-empty id and title":
    let episodes = extractEpisodes(apollo, WorkId)
    for ep in episodes:
      check ep.id.len > 0
      check ep.title.len > 0

  test "first episode id matches known episode":
    let episodes = extractEpisodes(apollo, WorkId)
    # The work page lists episodes; check one is present
    var found = false
    for ep in episodes:
      if ep.id == "1177354054882728956":
        found = true
        break
    check found

suite "parseEpisodePage":
  test "extracts body content from episode HTML":
    let result = parseEpisodePage(EpisodeHtml)
    check result.isSome
    let content = result.get
    check content.len > 0

  test "body contains paragraph text":
    let result = parseEpisodePage(EpisodeHtml)
    check result.isSome
    check "<p" in result.get

  test "returns none for HTML without .widget-episodeBody":
    let result = parseEpisodePage("<html><body><p>nothing</p></body></html>")
    check result.isNone
