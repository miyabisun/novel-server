import db_connector/db_sqlite
import std/[httpclient, locks]
import config, cache

type AppState* = ref object
  db*: DbConn
  dbLock*: Lock
  cache*: Cache
  config*: Config
  http*: AsyncHttpClient

proc newAppState*(db: DbConn, cache: Cache, config: Config): AppState =
  result = AppState(
    db: db,
    cache: cache,
    config: config,
    http: newAsyncHttpClient(
      userAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ),
  )
  initLock(result.dbLock)
