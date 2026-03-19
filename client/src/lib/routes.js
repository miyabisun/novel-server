export const routes = [
  { pattern: /^\/$/, params: [] },
  { pattern: /^\/ranking\/([^/]+)$/, params: ['type'] },
  { pattern: /^\/novel\/([^/]+)\/([^/]+)\/toc$/, params: ['type', 'id'] },
  { pattern: /^\/novel\/([^/]+)\/([^/]+)\/(\d+)$/, params: ['type', 'id', 'num'] },
]

export function matchRoute(path) {
  for (let i = 0; i < routes.length; i++) {
    const match = path.match(routes[i].pattern)
    if (match) {
      const params = {}
      routes[i].params.forEach((key, j) => {
        params[key] = decodeURIComponent(match[j + 1])
      })
      return { index: i, params }
    }
  }
  return { index: 0, params: {} }
}
