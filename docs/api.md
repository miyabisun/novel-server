# API リファレンス

すべての API エンドポイントは `/api` プレフィックス配下にあります。
`BASE_PATH` が設定されている場合は `{BASE_PATH}/api` になります。

---

## ランキング (Ranking)

### GET `/api/novel/:type/ranking`

指定サイトのランキングを取得します。結果は 3 時間キャッシュされます。

**パラメータ:**

| 名前 | 位置 | 値 | 説明 |
|------|------|-----|------|
| `type` | path | `narou` / `kakuyomu` / `nocturne` | 対象サイト |
| `period` | query | `daily` / `weekly` / `monthly` / `quarter` / `yearly` | 期間（デフォルト: `daily`） |

> **注意:** `kakuyomu` は `quarter` に非対応です（400 エラー）。

**レスポンス (200):**

```json
{
  "ジャンル名": [
    {
      "id": "n1234ab",
      "title": "小説タイトル",
      "page": 150,
      "noveltype": 1
    }
  ]
}
```

ジャンル名をキーとし、各ジャンルに小説の配列が入ったオブジェクトです。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `string` | 小説 ID |
| `title` | `string` | タイトル |
| `page` | `number` | 総ページ数 |
| `noveltype` | `number` | 1 = 連載, 2 = 短編 |

**エラー (400):**

```json
{ "error": "Invalid type" }
{ "error": "Invalid period" }
{ "error": "kakuyomu does not support quarter ranking" }
```

---

### PATCH `/api/novel/:type/ranking`

キャッシュを無視してランキングを再取得します。パラメータ・レスポンス形式は GET と同一です。

---

## 検索 (Search)

### GET `/api/novel/:type/search`

キーワードで小説を検索します。結果は 1 時間キャッシュされます。

**パラメータ:**

| 名前 | 位置 | 値 | 説明 |
|------|------|-----|------|
| `type` | path | `narou` / `kakuyomu` / `nocturne` | 対象サイト |
| `q` | query | 文字列 | 検索キーワード（必須） |

**レスポンス (200):**

```json
[
  {
    "id": "n1234ab",
    "title": "小説タイトル",
    "page": 150
  }
]
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `string` | 小説 ID |
| `title` | `string` | タイトル |
| `page` | `number` | 総ページ数 |

**エラー:**

```json
{ "error": "Invalid type" }          // 400
{ "error": "Missing query parameter: q" }  // 400
{ "error": "Failed to search" }      // 502
```

---

## 小説詳細 (Detail)

### GET `/api/novel/:type/:id/detail`

小説のタイトル・あらすじ・総ページ数を取得します。結果は 24 時間キャッシュされます。外部 API への取得は最大 3 回リトライします（500ms × 試行回数のバックオフ）。

**パラメータ:**

| 名前 | 型 | 説明 |
|------|-----|------|
| `type` | `string` | 対象サイト (`narou` / `kakuyomu` / `nocturne`) |
| `id` | `string` | 小説 ID |

**レスポンス (200):**

```json
{
  "title": "小説タイトル",
  "synopsis": "あらすじテキスト...",
  "page": 150
}
```

**エラー (502):**

```json
{ "error": "Failed to fetch detail" }
```

---

## ページ (Pages)

### GET `/api/novel/:type/:id/pages/:num`

小説の本文を取得します。結果は 24 時間キャッシュされます。外部 API への取得は最大 3 回リトライします（500ms × 試行回数のバックオフ）。

**パラメータ:**

| 名前 | 型 | 説明 |
|------|-----|------|
| `type` | `string` | 対象サイト (`narou` / `kakuyomu` / `nocturne`) |
| `id` | `string` | 小説 ID |
| `num` | `number` | ページ番号（1始まり） |

**レスポンス (200):**

```json
{
  "html": "<p>本文の HTML...</p>"
}
```

HTML は許可リスト方式でサニタイズ済みです（`p`, `br`, `div`, `span`, `ruby` 等のコンテンツタグのみ許可。許可外のタグと全属性は除去）。

---

### PATCH `/api/novel/:type/:id/pages/:num`

キャッシュを無視してページを再取得します。レスポンス形式は GET と同一です。

---

## お気に入り (Favorites)

### GET `/api/favorites`

お気に入り一覧を小説更新日時の降順で取得します（更新日時のないものは末尾）。

**レスポンス (200):**

```json
[
  {
    "type": "narou",
    "id": "n1234ab",
    "title": "小説タイトル",
    "novelupdated_at": "2026-02-15T00:00:00",
    "page": 150,
    "read": 42
  }
]
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `type` | `string` | サイト種別 |
| `id` | `string` | 小説 ID |
| `title` | `string` | 小説タイトル |
| `novelupdated_at` | `string?` | 小説の更新日時（nullable） |
| `page` | `number` | 総ページ数 |
| `read` | `number` | 既読ページ番号（0 = 未読） |

---

### PUT `/api/favorites/:type/:id`

お気に入りを追加または更新します（upsert）。

**パラメータ:**

| 名前 | 型 | 説明 |
|------|-----|------|
| `type` | `string` | サイト種別 |
| `id` | `string` | 小説 ID |

**リクエスト:**

```json
{
  "title": "小説タイトル",
  "page": 150,
  "novelupdated_at": "2026-02-15T00:00:00"
}
```

`novelupdated_at` は省略可能です。

**レスポンス (200):** 作成/更新された Favorite オブジェクト。

---

### DELETE `/api/favorites/:type/:id`

お気に入りを削除します。

**レスポンス (200):**

```json
{ "ok": true }
```

---

### PATCH `/api/favorites/:type/:id/progress`

既読位置を更新します。お気に入りに登録されていない場合は何もせず `{ "ok": true }` を返します。

**リクエスト:**

```json
{
  "read": 42
}
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `read` | `number` | 既読ページ番号 |

**レスポンス (200):** 更新された Favorite オブジェクト。お気に入り未登録の場合は `{ "ok": true }`。
