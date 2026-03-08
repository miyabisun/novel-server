# Package
version       = "0.1.0"
author        = "miyabisun"
description   = "Novel ranking viewer & reader"
license       = "MIT"
srcDir        = "src"
bin           = @["main"]

# Dependencies
requires "nim >= 2.0.0"
requires "jester >= 0.6.0"
requires "nimquery >= 2.0.1"
requires "dotenv >= 2.0.0"
requires "db_connector >= 0.1.0"

task test, "Run unit tests":
  exec "nim c -r test/t_sanitize.nim"
  exec "nim c -r test/t_syosetu.nim"
  exec "nim c -r -d:ssl test/t_kakuyomu.nim"
