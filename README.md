# novel-server

なろう・カクヨム・ノクターンに対応した小説ランキングビューア＆リーダー。
お気に入り管理と既読位置の自動記録機能付き。

## 技術スタック

- **バックエンド**: Hono + TypeScript + Prisma (SQLite)
- **フロントエンド**: Svelte 5 + Vite + Sass
- **認証**: JWT (HS256) + HttpOnly Cookie

## セットアップ

```bash
npm install
cd client && npm install && cd ..
```

### 環境変数

`.env` を作成して以下を設定（全て必須）:

```env
PORT=3000
DATABASE_URL="file:./novel.db"
AUTH_USERNAME=admin
AUTH_PASSWORD=<パスワード>
JWT_SECRET=<ランダムな秘密鍵（32文字以上推奨）>
```

`AUTH_USERNAME`、`AUTH_PASSWORD`、`JWT_SECRET` が未設定の場合、サーバーは起動時にエラーで停止します。

### データベース

```bash
npm run db:push
```

## 起動

```bash
# 開発（フロントエンドのホットリビルド付き）
npm run dev

# 本番
npm run build
npm start
```

`http://localhost:3000` にアクセスするとログイン画面が表示されます。

## 主な機能

- **ランキング閲覧** — なろう / カクヨム / ノクターンの日間ランキングをタブ切替で表示
- **小説リーダー** — キーボード（矢印キー）対応のページ送り
- **お気に入り** — ランキングから★で追加、一覧から管理・削除
- **既読位置記録** — リーダーでページ読み込み成功時に自動で進捗を保存
- **認証** — 全 API がログイン必須（シングルユーザー）

## npm scripts

| コマンド | 説明 |
|---------|------|
| `npm run dev` | 開発サーバー起動（vite watch + node） |
| `npm run build` | フロントエンドビルド |
| `npm start` | 本番サーバー起動 |
| `npm run db:push` | Prisma スキーマを DB に同期 |

## リバースプロキシ配下での運用

`BASE_PATH` を設定するとサブパスで動作します:

```env
BASE_PATH=/novels
```

## ドキュメント

- [API リファレンス](docs/api.md)
- [認証の仕組み](docs/authentication.md)
- [アーキテクチャ](docs/architecture.md)
