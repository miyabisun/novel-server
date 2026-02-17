# API リファレンス

すべての API エンドポイントは `/api` プレフィックス配下にあります。
`BASE_PATH` が設定されている場合は `{BASE_PATH}/api` になります。

認証が必要なエンドポイントには `novel_token` Cookie（JWT）が必要です。
未認証の場合は `401 Unauthorized` が返ります。

---

## 認証 (Auth)

### POST `/api/auth/login`

ログインして JWT トークンを取得します。**認証不要**。

**リクエスト:**

```json
{
  "username": "admin",
  "password": "changeme"
}
```

**レスポンス (成功 200):**

```json
{ "ok": true }
```

`novel_token` が HttpOnly Cookie としてセットされます（有効期限: 7日間）。

**レスポンス (失敗 401):**

```json
{ "error": "Invalid credentials" }
```

---

### POST `/api/auth/logout`

Cookie を削除してログアウトします。

**レスポンス:**

```json
{ "ok": true }
```

---

### GET `/api/auth/me`

現在の認証状態を確認します。JWT ミドルウェアを通過した場合のみ到達するため、レスポンスが返れば認証済みです。

**レスポンス (200):**

```json
{ "authenticated": true }
```

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
      "page": 150
    }
  ]
}
```

ジャンル名をキーとし、各ジャンルに小説の配列が入ったオブジェクトです。

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

## ページ (Pages)

### GET `/api/novel/:type/:id/pages/:num`

小説の本文を取得します。結果は 24 時間キャッシュされます。

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

HTML はサニタイズ済みです（`<script>`, `<iframe>`, `on*` イベント属性は除去）。

---

### PATCH `/api/novel/:type/:id/pages/:num`

キャッシュを無視してページを再取得します。レスポンス形式は GET と同一です。

---

## お気に入り (Favorites)

### GET `/api/favorites`

お気に入り一覧をタイトル昇順で取得します。

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

既読位置を更新します。お気に入りに登録されていない場合は 404 を返します。

**リクエスト:**

```json
{
  "read": 42
}
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `read` | `number` | 既読ページ番号 |

**レスポンス (200):** 更新された Favorite オブジェクト。

**レスポンス (404):**

```json
{ "error": "Not found" }
```

リーダーはページ表示時にこのエンドポイントを叩きますが、お気に入り未登録の場合の 404 は無視されます。
