import { describe, expect, test } from 'bun:test'
import { mapItem, parsePage } from './syosetu.js'

describe('mapItem — 意地悪なAPI応答', () => {
  test('ncode が null の場合にクラッシュしない', () => {
    // なろうAPIが削除済み小説で null を返すケース
    expect(() => mapItem({ ncode: null })).not.toThrow()
  })

  test('title が null の場合にクラッシュしない', () => {
    // 非公開小説や API 障害時に title が null になるケース
    expect(() => mapItem({ title: null })).not.toThrow()
  })

  test('ncode が数値の場合にクラッシュしない', () => {
    // API のレスポンスが想定外の型を返すケース
    expect(() => mapItem({ ncode: 12345 })).not.toThrow()
  })

  test('title が数値の場合にクラッシュしない', () => {
    // 数値のみのタイトルが number 型で返されるケース
    expect(() => mapItem({ title: 0 })).not.toThrow()
  })
})

describe('parsePage — 意地悪なHTML', () => {
  test('セレクタに一致するが中身がホワイトスペースのみの場合', () => {
    // サイト構造変更で本文が空白だけになったケース
    const html = '<div id="novel_honbun">   \n\t  </div>'
    const result = parsePage(html, '#novel_honbun')
    // 空白のみなら null を返すべき（読者に見せる中身がない）
    expect(result).toBeNull()
  })
})
