import std/[os, json, strutils, times, options]

var
  cachedHtml: string = ""
  cachedMtime: Time = Time()

proc getIndexHtml*(basePath: string): Option[string] =
  let indexPath = "client/build/index.html"

  if not fileExists(indexPath):
    return none(string)

  let isProd = getEnv("NODE_ENV") == "production"
  let info = getFileInfo(indexPath)
  let mtime = info.lastWriteTime

  if cachedHtml.len > 0:
    if isProd or mtime == cachedMtime:
      return some(cachedHtml)

  let raw = readFile(indexPath)
  var html = raw.replace("<head>",
    "<head>\n\t\t<base href=\"" & basePath & "/\">")
  html = html.replace(
    "window.__BASE_PATH__ = \"\"",
    "window.__BASE_PATH__ = " & $(%basePath))

  cachedHtml = html
  cachedMtime = mtime
  some(html)
