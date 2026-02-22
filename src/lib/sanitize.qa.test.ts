import { describe, expect, test } from 'bun:test'
import { sanitizeHtml } from './sanitize.js'

describe('sanitizeHtml — 意地悪な入力', () => {
  test('HTMLコメントを除去する', () => {
    // IE条件付きコメントやコメント内のスクリプト注入
    // <!--[if IE]><script>alert(1)</script><![endif]-->
    expect(sanitizeHtml('hello<!-- evil -->world')).toBe('helloworld')
  })

  test('<style>タグの中身がテキストとして漏れない', () => {
    // <style> が removeAndKeepContent されると CSS がそのまま表示される
    expect(sanitizeHtml('<style>*{display:none}</style>')).toBe('')
  })

  test('<title>タグの中身がテキストとして漏れない', () => {
    // 同様に <title> のテキストが意図せず混入する
    expect(sanitizeHtml('<title>悪意のあるタイトル</title>')).toBe('')
  })

  test('大文字のタグ名でもサニタイズされる', () => {
    // HTMLRewriter がタグ名を正規化するか確認
    expect(sanitizeHtml('<SCRIPT>alert(1)</SCRIPT>')).toBe('')
  })
})
