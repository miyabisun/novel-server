import std/[tables, times, json, options, locks]

const
  MaxEntries = 10_000
  SweepIntervalSecs = 3600

type
  CacheEntry = object
    value: JsonNode
    expiresAt: Option[DateTime]

  Cache* = ref object
    store: Table[string, CacheEntry]
    lock: Lock

proc newCache*(): Cache =
  result = Cache(store: initTable[string, CacheEntry]())
  initLock(result.lock)

proc get*(c: Cache, key: string): Option[JsonNode] =
  acquire(c.lock)
  defer: release(c.lock)
  if key notin c.store:
    return none(JsonNode)
  let entry = c.store[key]
  if entry.expiresAt.isSome:
    if now() > entry.expiresAt.get:
      return none(JsonNode)
  some(entry.value)

proc put*(c: Cache, key: string, value: JsonNode, ttlSeconds: int = 0) =
  acquire(c.lock)
  defer: release(c.lock)
  if c.store.len >= MaxEntries and key notin c.store:
    # Remove first entry (oldest inserted)
    for k in c.store.keys:
      c.store.del(k)
      break
  let expiresAt = if ttlSeconds > 0:
    some(now() + initDuration(seconds = ttlSeconds))
  else:
    none(DateTime)
  c.store[key] = CacheEntry(value: value, expiresAt: expiresAt)

proc sweep*(c: Cache) =
  acquire(c.lock)
  defer: release(c.lock)
  let current = now()
  var toDelete: seq[string]
  for key, entry in c.store:
    if entry.expiresAt.isSome and current > entry.expiresAt.get:
      toDelete.add(key)
  for key in toDelete:
    c.store.del(key)

proc sweepIntervalSecs*(): int = SweepIntervalSecs
