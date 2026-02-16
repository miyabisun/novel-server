# 認証の仕組み

## 概要

novel-server はシングルユーザー認証を採用しています。
ユーザー名とパスワードは `.env` に直接定義し、データベースにユーザーテーブルは持ちません。

## 認証フロー

```
ブラウザ                        サーバー
  │                               │
  ├── POST /api/auth/login ──────►│ username/password を .env の値と比較
  │   { username, password }      │
  │                               │ 一致 → JWT 生成
  │◄── Set-Cookie: novel_token ───┤ HttpOnly Cookie にセット
  │                               │
  ├── GET /api/* ────────────────►│ Cookie から JWT を検証
  │   Cookie: novel_token=xxx     │
  │◄── 200 レスポンス ─────────────┤
  │                               │
  ├── POST /api/auth/logout ─────►│ Cookie 削除
  │◄── Set-Cookie: (削除) ────────┤
  │                               │
```

## JWT トークン

- **ライブラリ**: `hono/jwt`（Hono 組み込み）
- **ペイロード**: `{ sub: username, exp: <7日後のUNIXタイムスタンプ> }`
- **署名鍵**: `.env` の `JWT_SECRET`

## Cookie 設定

| 属性 | 値 | 理由 |
|------|-----|------|
| `httpOnly` | `true` | JavaScript からのアクセスを防止（XSS 対策） |
| `sameSite` | `Lax` | CSRF 対策。通常のナビゲーションでは送信される |
| `path` | `/` | 全パスで有効 |
| `maxAge` | 604800 (7日) | トークンの有効期限と一致 |

## ミドルウェアの適用

`src/index.ts` で `/api/*` パスに JWT ミドルウェアを適用しています。

```
/api/auth/login    → ミドルウェアをスキップ（ログイン前なので）
/api/auth/logout   → JWT 検証あり
/api/auth/me       → JWT 検証あり
/api/favorites/*   → JWT 検証あり
/api/novel/*       → JWT 検証あり
```

## フロントエンドの認証制御

SPA の静的ファイル（HTML/JS/CSS）は認証なしで配信されます。
認証の制御はフロントエンド側で行います。

1. **起動時**: `checkAuth()` が `GET /api/auth/me` を叩いて認証状態を確認
2. **未認証**: `/login` 以外のルートにいる場合、`/login` にリダイレクト
3. **認証済み**: `/login` にアクセスした場合、`/` にリダイレクト
4. **401 応答**: `fetcher.js` が 401 を検知すると自動で `/login` にリダイレクト

## 環境変数

| 変数 | 説明 | 例 |
|------|------|-----|
| `AUTH_USERNAME` | ログインユーザー名 | `admin` |
| `AUTH_PASSWORD` | ログインパスワード | 任意の強力なパスワード |
| `JWT_SECRET` | JWT 署名鍵 | ランダムな文字列（32文字以上推奨） |
