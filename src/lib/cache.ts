interface CacheEntry {
  value: unknown
  expiresAt: number | null
}

const store = new Map<string, CacheEntry>()

export default {
  get(key: string): unknown | null {
    const entry = store.get(key)
    if (!entry) return null
    if (entry.expiresAt && Date.now() > entry.expiresAt) {
      store.delete(key)
      return null
    }
    return entry.value
  },

  set(key: string, value: unknown, ttlSeconds?: number): void {
    store.set(key, {
      value,
      expiresAt: ttlSeconds ? Date.now() + ttlSeconds * 1000 : null,
    })
  },

  del(key: string): void {
    store.delete(key)
  },
}
