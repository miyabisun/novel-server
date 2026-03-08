import std/json

type
  AppErrorKind* = enum
    BadRequest, NotFound, Upstream, Internal

  AppError* = object of CatchableError
    kind*: AppErrorKind

proc newAppError*(kind: AppErrorKind, msg: string): ref AppError =
  result = newException(AppError, msg)
  result.kind = kind

proc statusCode*(kind: AppErrorKind): int =
  case kind
  of BadRequest: 400
  of NotFound: 404
  of Upstream: 502
  of Internal: 500

proc toJson*(e: ref AppError): JsonNode =
  %*{"error": e.msg}
