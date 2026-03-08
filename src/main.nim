import std/[asyncdispatch, json, strutils, logging, os]
import jester
import config, db, cache, state, error, spa, sync
import routes/[router]

# Load .env file
proc loadDotenv() =
  let envFile = ".env"
  if fileExists(envFile):
    for line in lines(envFile):
      let stripped = line.strip
      if stripped.len == 0 or stripped.startsWith("#"):
        continue
      let eqIdx = stripped.find('=')
      if eqIdx > 0:
        let key = stripped[0 ..< eqIdx].strip
        let val = stripped[eqIdx + 1 .. ^1].strip
        if key.len > 0 and getEnv(key).len == 0:
          putEnv(key, val)

proc main() =
  addHandler(newConsoleLogger(fmtStr = "$datetime [$levelname] "))

  loadDotenv()

  let cfg = loadConfig()
  let conn = db.open(cfg.dbPath)
  let appCache = newCache()
  let appState = newAppState(conn, appCache, cfg)

  # Start cache sweep timer
  proc sweepLoop() {.async.} =
    while true:
      await sleepAsync(sweepIntervalSecs() * 1000)
      appCache.sweep()

  asyncCheck sweepLoop()

  # Start background sync
  startSync(appState)

  let basePath = cfg.basePath
  let displayPath = if basePath.len == 0: "/" else: basePath

  info "Server running on http://localhost:" & $cfg.port & displayPath

  let settings = newSettings(
    port = Port(cfg.port),
    bindAddr = "0.0.0.0",
  )

  var jesterApp = initJester(buildRouter(appState), settings)
  jesterApp.serve()

when isMainModule:
  main()
