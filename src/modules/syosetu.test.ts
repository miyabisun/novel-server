import { describe, expect, test } from 'bun:test'
import { mapItem, buildPages, parsePage } from './syosetu.js'

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
})
