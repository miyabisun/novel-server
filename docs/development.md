# 開発ガイド

## 技術スタック

- **バックエンド**: Bun + Hono + TypeScript + Drizzle ORM (SQLite)
- **フロントエンド**: Svelte 5 + Vite + Sass

## セットアップ

```bash
bun run setup
```

このコマンドで以下が実行されます:

1. サーバー側の依存関係をインストール (`bun install`)
2. クライアント側の依存関係をインストール (`cd client && bun install`)
3. `.env` が存在しなければ `.env.example` からコピー

`.env` を手動で作成する場合は `.env.example` を参考にしてください。

## 起動

```bash
# 開発（フロントエンドのホットリビルド付き）
bun run dev

# 本番
bun run build
bun start
```

`http://localhost:3000` にアクセスするとアプリケーションが表示されます。

> **注意:** `bun run dev` はフロントエンドの自動リビルド (vite --watch) のみ行います。サーバー側のコード (`src/`) を変更した場合は手動で再起動してください。

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `bun run setup` | 初期セットアップ（依存インストール + .env 生成） |
| `bun run dev` | 開発サーバー起動（vite watch + bun） |
| `bun run build` | フロントエンドビルド |
| `bun run build:client` | フロントエンドの依存インストール + ビルド |
| `bun start` | 本番サーバー起動 |
| `bun run test` | テスト実行 |

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

## リバースプロキシ

### BASE_PATH なし（ルート配信）

```nginx
server {
    listen 80;
    server_name novels.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

### BASE_PATH あり（サブパス配信）

`.env` で `BASE_PATH=/novels` を設定した上で:

```nginx
location /novels {
    proxy_pass http://localhost:3000/novels;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}
```

## SQLite 運用

### Volume マウントの注意点

SQLite は WAL モードで動作するため、DB ファイルと同じディレクトリに `*.db-wal` と `*.db-shm` が生成される。Volume マウントはファイル単位ではなくディレクトリ単位で行うこと。

```bash
# 正しい: ディレクトリをマウント
docker run -v novel-data:/data novel-server

# 誤り: ファイルだけマウントすると WAL/SHM が失われる
docker run -v ./novel.db:/data/novel.db novel-server
```

### バックアップ

稼働中のバックアップは SQLite の `.backup` コマンドを使う。ファイルコピーは WAL が未フラッシュの場合にデータ不整合を起こす可能性がある。

```bash
sqlite3 /data/novel.db ".backup /backup/novel-$(date +%Y%m%d).db"
```
