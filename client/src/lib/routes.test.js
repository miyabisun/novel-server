import { describe, it, expect } from 'vitest'
import { routes, matchRoute } from './routes.js'

describe('routes', () => {
  it('has 4 route definitions', () => {
    expect(routes).toHaveLength(4)
  })
})

describe('matchRoute', () => {
  it('matches root path to index 0 (Favorites)', () => {
    const result = matchRoute('/')
    expect(result).toEqual({ index: 0, params: {} })
  })

  it('matches ranking path to index 1', () => {
    const result = matchRoute('/ranking/narou')
    expect(result).toEqual({ index: 1, params: { type: 'narou' } })
  })

  it('matches ranking with different types', () => {
    for (const type of ['narou', 'kakuyomu', 'nocturne']) {
      const result = matchRoute(`/ranking/${type}`)
      expect(result.index).toBe(1)
      expect(result.params.type).toBe(type)
    }
  })

  it('matches toc path to index 2', () => {
    const result = matchRoute('/novel/narou/n1234ab/toc')
    expect(result).toEqual({
      index: 2,
      params: { type: 'narou', id: 'n1234ab' },
    })
  })

  it('matches reader path to index 3', () => {
    const result = matchRoute('/novel/kakuyomu/abc123/42')
    expect(result).toEqual({
      index: 3,
      params: { type: 'kakuyomu', id: 'abc123', num: '42' },
    })
  })

  it('reader path requires numeric page number', () => {
    const result = matchRoute('/novel/narou/n1234ab/notanumber')
    // Should not match reader (index 3) since page is not digits
    expect(result.index).not.toBe(3)
  })

  it('falls back to index 0 for unknown paths', () => {
    const result = matchRoute('/unknown/path')
    expect(result).toEqual({ index: 0, params: {} })
  })

  it('falls back to index 0 for empty path', () => {
    const result = matchRoute('')
    expect(result).toEqual({ index: 0, params: {} })
  })

  it('decodes URI-encoded params', () => {
    const result = matchRoute('/ranking/%E3%83%86%E3%82%B9%E3%83%88')
    expect(result.index).toBe(1)
    expect(result.params.type).toBe('テスト')
  })

  it('does not match trailing slash on ranking', () => {
    const result = matchRoute('/ranking/narou/')
    expect(result.index).toBe(0) // no match, falls back
  })

  it('does not match extra segments on toc', () => {
    const result = matchRoute('/novel/narou/n1234ab/toc/extra')
    expect(result.index).toBe(0)
  })
})
