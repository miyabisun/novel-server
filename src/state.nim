import db_connector/db_sqlite
import std/locks
import config, cache

type AppState* = ref object
  db*: DbConn
  dbLock*: Lock
  cache*: Cache
  config*: Config

proc newAppState*(db: DbConn, cache: Cache, config: Config): AppState =
  result = AppState(
    db: db,
    cache: cache,
    config: config,
  )
  initLock(result.dbLock)
