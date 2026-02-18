# 開発ガイド

## 技術スタック

- **バックエンド**: Hono + TypeScript + Prisma (SQLite)
- **フロントエンド**: Svelte 5 + Vite + Sass

## セットアップ

```bash
npm run setup
```

このコマンドで以下が実行されます:

1. サーバー側の依存関係をインストール (`npm install`)
2. クライアント側の依存関係をインストール (`cd client && npm install`)
3. `.env` が存在しなければ `.env.example` からコピー

`.env` を手動で作成する場合は `.env.example` を参考にしてください。

## 起動

```bash
# 開発（フロントエンドのホットリビルド付き）
npm run dev

# 本番
npm run build
npm start
```

`http://localhost:3000` にアクセスするとアプリケーションが表示されます。

> **注意:** `npm run dev` はフロントエンドの自動リビルド (vite --watch) のみ行います。サーバー側のコード (`src/`) を変更した場合は手動で再起動してください。

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `npm run setup` | 初期セットアップ（依存インストール + .env 生成） |
| `npm run dev` | 開発サーバー起動（vite watch + node） |
| `npm run build` | フロントエンドビルド |
| `npm run build:client` | フロントエンドの依存インストール + ビルド |
| `npm start` | 本番サーバー起動 |

## 環境変数

`.env.example` の内容:

```env
PORT=3000
DATABASE_PATH=./novel.db
# BASE_PATH=/novels
```

| 環境変数 | デフォルト | 説明 |
|---|---|---|
| `DATABASE_PATH` | `./novel.db` | SQLite データベースファイルのパス（Docker 環境では `/data/novel.db`） |
| `PORT` | `3000` | サーバーのポート番号 |
| `BASE_PATH` | (なし) | リバースプロキシ配下で使う場合のパス |

## Docker ビルド

```bash
docker build -t novel-server .
docker run -p 3000:3000 -v novel-data:/data novel-server
```
