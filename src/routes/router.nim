import std/[asyncdispatch, json, strutils, os, options, httpcore]
import jester
import ../state, ../spa
import ranking, search, detail, toc, pages, favorites

proc buildRouter*(state: AppState): auto =
  let basePath = state.config.basePath

  router appRouter:
    get "/api/novel/@type/ranking":
      let period = if request.params.hasKey("period"): request.params["period"] else: "daily"
      let (code, body) = await handleRanking(state, @"type", period, true)
      resp code, {"Content-Type": "application/json"}, body

    patch "/api/novel/@type/ranking":
      let period = if request.params.hasKey("period"): request.params["period"] else: "daily"
      let (code, body) = await handleRanking(state, @"type", period, false)
      resp code, {"Content-Type": "application/json"}, body

    get "/api/novel/@type/search":
      let q = if request.params.hasKey("q"): request.params["q"] else: ""
      let (code, body) = await handleSearch(state, @"type", q)
      resp code, {"Content-Type": "application/json"}, body

    get "/api/novel/@type/@id/detail":
      let (code, body) = await handleDetail(state, @"type", @"id")
      resp code, {"Content-Type": "application/json"}, body

    get "/api/novel/@type/@id/toc":
      let (code, body) = await handleToc(state, @"type", @"id")
      resp code, {"Content-Type": "application/json"}, body

    get "/api/novel/@type/@id/pages/@num":
      let (code, body) = await handleGetPage(state, @"type", @"id", @"num")
      resp code, {"Content-Type": "application/json"}, body

    patch "/api/novel/@type/@id/pages/@num":
      let (code, body) = await handlePatchPage(state, @"type", @"id", @"num")
      resp code, {"Content-Type": "application/json"}, body

    get "/api/favorites":
      let (code, body) = await handleGetFavorites(state)
      resp code, {"Content-Type": "application/json"}, body

    put "/api/favorites/@type/@id":
      let (code, body) = await handlePutFavorite(state, request.body, @"type", @"id")
      resp code, {"Content-Type": "application/json"}, body

    delete "/api/favorites/@type/@id":
      let (code, body) = await handleDeleteFavorite(state, @"type", @"id")
      resp code, {"Content-Type": "application/json"}, body

    patch "/api/favorites/@type/@id/progress":
      let (code, body) = await handlePatchProgress(state, request.body, @"type", @"id")
      resp code, {"Content-Type": "application/json"}, body

    get "/assets/@file":
      let filePath = "client/build/assets/" & @"file"
      if fileExists(filePath):
        sendFile(filePath)
      else:
        resp Http404, "Not found"

    get "/favicon.svg":
      let filePath = "client/build/favicon.svg"
      if fileExists(filePath):
        sendFile(filePath)
      else:
        resp Http404, "Not found"

    error Http404:
      let html = getIndexHtml(basePath)
      if html.isSome:
        resp Http200, {"Content-Type": "text/html"}, html.get
      else:
        resp Http404, {"Content-Type": "application/json"},
          $(%*{"error": "Frontend not built. Run: cd client && npm install && npx vite build"})

  appRouter
