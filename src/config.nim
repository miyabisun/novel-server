import std/[os, strutils, re]

type Config* = object
  port*: int
  basePath*: string
  dbPath*: string

proc loadConfig*(): Config =
  let port = try:
    parseInt(getEnv("PORT", "3000"))
  except ValueError:
    3000

  var basePath = getEnv("BASE_PATH", "").strip(chars = {'/'}, leading = false)
  if basePath.len > 0:
    if not basePath.match(re"^/[\w\-/]*$"):
      raise newException(ValueError, "Invalid BASE_PATH: " & basePath)

  let dbPath = getEnv("DATABASE_PATH", "/data/novel.db")

  Config(port: port, basePath: basePath, dbPath: dbPath)
