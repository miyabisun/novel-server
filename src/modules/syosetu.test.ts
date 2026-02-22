import { describe, expect, test } from 'bun:test'
import { mapItem, buildPages, parsePage, parseToc } from './syosetu.js'

describe('mapItem', () => {
  test('maps ncode to lowercase id', () => {
    const result = mapItem({ ncode: 'N1234AB' })
    expect(result.id).toBe('n1234ab')
  })

  test('trims title whitespace', () => {
    const result = mapItem({ title: '  テスト小説  ' })
    expect(result.title).toBe('テスト小説')
  })

  test('maps general_all_no to page', () => {
    const result = mapItem({ general_all_no: 150 })
    expect(result.page).toBe(150)
  })

  test('passes through other keys unchanged', () => {
    const result = mapItem({ novelupdated_at: '2026-01-01', genre: 201 })
    expect(result.novelupdated_at).toBe('2026-01-01')
    expect(result.genre).toBe(201)
  })

  test('maps a complete API response item', () => {
    const result = mapItem({
      ncode: 'N9999ZZ',
      title: '  異世界転生  ',
      general_all_no: 42,
      novelupdated_at: '2026-02-22',
    })
    expect(result).toEqual({
      id: 'n9999zz',
      title: '異世界転生',
      page: 42,
      novelupdated_at: '2026-02-22',
    })
  })

  test('handles null ncode without crashing', () => {
    expect(() => mapItem({ ncode: null })).not.toThrow()
  })

  test('handles null title without crashing', () => {
    expect(() => mapItem({ title: null })).not.toThrow()
  })

  test('handles numeric ncode without crashing', () => {
    expect(() => mapItem({ ncode: 12345 })).not.toThrow()
  })

  test('handles numeric title without crashing', () => {
    expect(() => mapItem({ title: 0 })).not.toThrow()
  })
})

describe('buildPages', () => {
  test('generates page entries for given count', () => {
    const pages = buildPages('narou', 'n1234ab', 3)
    expect(pages).toEqual([
      { type: 'narou', id: 'n1234ab', num: 1, page_id: '1' },
      { type: 'narou', id: 'n1234ab', num: 2, page_id: '2' },
      { type: 'narou', id: 'n1234ab', num: 3, page_id: '3' },
    ])
  })

  test('returns empty array for count 0', () => {
    expect(buildPages('narou', 'n1234ab', 0)).toEqual([])
  })
})

describe('parsePage', () => {
  test('extracts content matching selector', () => {
    const html = '<div id="novel_honbun"><p>第一段落</p><p>第二段落</p></div>'
    const result = parsePage(html, '#novel_honbun')
    expect(result).toBe('<p>第一段落</p><p>第二段落</p>')
  })

  test('joins multiple matches with <hr>', () => {
    const html = '<div class="part"><p>前編</p></div><div class="part"><p>後編</p></div>'
    const result = parsePage(html, '.part')
    expect(result).toBe('<p>前編</p><hr><p>後編</p>')
  })

  test('returns null when selector matches nothing', () => {
    const html = '<div>content</div>'
    const result = parsePage(html, '.nonexistent')
    expect(result).toBeNull()
  })

  test('returns null for empty matched content', () => {
    const html = '<div class="target"></div>'
    const result = parsePage(html, '.target')
    expect(result).toBeNull()
  })

  test('returns null for whitespace-only content', () => {
    const html = '<div id="novel_honbun">   \n\t  </div>'
    const result = parsePage(html, '#novel_honbun')
    expect(result).toBeNull()
  })
})

describe('parseToc', () => {
  test('extracts title and episodes from TOC page', () => {
    const html = `
      <h1 class="p-novel__title">テスト小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話 始まり</a></div>
      <div class="p-eplist__sublist"><a href="/n1234ab/2/">第2話 展開</a></div>
    `
    const result = parseToc(html)
    expect(result).toEqual({
      title: 'テスト小説',
      episodes: [
        { num: 1, title: '第1話 始まり' },
        { num: 2, title: '第2話 展開' },
      ],
      lastPage: 1,
    })
  })

  test('falls back to <title> when .p-novel__title is missing', () => {
    const html = `
      <html><head><title>フォールバックタイトル</title></head>
      <body>
        <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
      </body></html>
    `
    const result = parseToc(html)
    expect(result.title).toBe('フォールバックタイトル')
    expect(result.episodes).toHaveLength(1)
  })

  test('returns empty episodes when no .p-eplist__sublist found', () => {
    const html = '<h1 class="p-novel__title">短編小説</h1><div>本文</div>'
    const result = parseToc(html)
    expect(result).toEqual({ title: '短編小説', episodes: [], lastPage: 1 })
  })

  test('trims whitespace from title and episode titles', () => {
    const html = `
      <h1 class="p-novel__title">  スペース付き  </h1>
      <div class="p-eplist__sublist"><a>  第1話  </a></div>
    `
    const result = parseToc(html)
    expect(result.title).toBe('スペース付き')
    expect(result.episodes[0].title).toBe('第1話')
  })

  test('extracts lastPage from pagination link', () => {
    const html = `
      <h1 class="p-novel__title">長編小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
      <a href="/n1234ab/?p=5">最後へ</a>
    `
    const result = parseToc(html)
    expect(result.lastPage).toBe(5)
  })

  test('returns lastPage 1 when no pagination', () => {
    const html = `
      <h1 class="p-novel__title">短め小説</h1>
      <div class="p-eplist__sublist"><a href="/n1234ab/1/">第1話</a></div>
    `
    const result = parseToc(html)
    expect(result.lastPage).toBe(1)
  })
})
