import { describe, expect, test } from 'bun:test'
import { sanitizeHtml } from './sanitize.js'

describe('sanitizeHtml', () => {
  test('returns empty string for null input', () => {
    expect(sanitizeHtml(null)).toBe('')
  })

  test('returns empty string for empty string input', () => {
    expect(sanitizeHtml('')).toBe('')
  })

  test('preserves allowed tags', () => {
    expect(sanitizeHtml('<p>hello</p>')).toBe('<p>hello</p>')
    expect(sanitizeHtml('<strong>bold</strong>')).toBe('<strong>bold</strong>')
    expect(sanitizeHtml('<ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby>'))
      .toBe('<ruby>漢<rt>かん</rt>字<rt>じ</rt></ruby>')
  })

  test('strips all attributes from allowed tags', () => {
    expect(sanitizeHtml('<p class="foo" id="bar">text</p>')).toBe('<p>text</p>')
    expect(sanitizeHtml('<div style="color:red">text</div>')).toBe('<div>text</div>')
  })

  test('removes script tags with content (XSS prevention)', () => {
    expect(sanitizeHtml('<script>alert("xss")</script>')).toBe('')
  })

  test('removes event handler attributes (XSS prevention)', () => {
    expect(sanitizeHtml('<p onclick="alert(1)">text</p>')).toBe('<p>text</p>')
  })

  test('removes img tags with onerror (XSS prevention)', () => {
    expect(sanitizeHtml('<img src=x onerror="alert(1)">')).toBe('')
  })

  test('removes non-allowed tags but keeps content', () => {
    expect(sanitizeHtml('<a href="http://example.com">link</a>')).toBe('link')
    expect(sanitizeHtml('<font color="red">text</font>')).toBe('text')
  })

  test('handles nested tags', () => {
    expect(sanitizeHtml('<p><strong>bold <em>italic</em></strong></p>'))
      .toBe('<p><strong>bold <em>italic</em></strong></p>')
  })

  test('handles mixed allowed and disallowed tags', () => {
    expect(sanitizeHtml('<p><script>bad</script>good</p>'))
      .toBe('<p>good</p>')
  })
})
