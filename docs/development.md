# 開発ガイド

## 技術スタック

- **バックエンド**: Rust (axum + tokio + rusqlite)
- **フロントエンド**: Svelte 5 + Vite + Sass

## 前提条件

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (フロントエンドビルド用)

## セットアップ

```bash
# フロントエンドの依存インストール + ビルド
cd client && npm install && npx vite build && cd ..

# .env を作成
cp .env.example .env
```

## 起動

```bash
# 開発
cargo run

# 本番（リリースビルド）
cargo run --release

# または bin/dev で一括ビルド＆起動
bin/dev
```

`http://localhost:3000` にアクセスするとアプリケーションが表示されます。

> **注意:** フロントエンドの変更時は `cd client && npx vite build` でリビルドしてください。

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `cd client && npm install && npx vite build` | フロントエンドセットアップ + ビルド |
| `cargo run` | 開発サーバー起動 |
| `cargo run --release` | 本番サーバー起動 |
| `cargo build --release` | リリースビルド |
| `cargo test` | テスト実行 |

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
