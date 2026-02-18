# novel-server

なろう・カクヨム・ノクターンに対応した小説ランキングビューア＆リーダーです。

お気に入り管理と既読位置の自動記録機能付き。

## Quick Start (Docker)

```bash
docker run -p 3000:3000 -v novel-data:/data ghcr.io/miyabisun/novel-server:latest
```

ブラウザで `http://localhost:3000` を開く。

## Quick Start (Node.js)

```bash
npm run setup && npm run build:client
npm start
```

ブラウザで `http://localhost:3000` を開く。

> リバースプロキシのサブパス配下にデプロイする場合は `BASE_PATH` 環境変数を設定してください。

## 設定

| 環境変数 | デフォルト | 説明 |
|---|---|---|
| `DATABASE_PATH` | `/data/novel.db` | SQLite データベースファイルのパス |
| `PORT` | `3000` | サーバーのポート番号 |
| `BASE_PATH` | (なし) | リバースプロキシ配下で使う場合のパス (例: `/novels`)。ランタイム設定のみで再ビルド不要。 |

データベースは初回起動時に自動生成されます。

## 主な機能

- **ランキング閲覧** — なろう / カクヨム / ノクターンのランキングを期間選択で表示、あらすじモーダル付き
- **小説リーダー** — キーボード（矢印キー）対応のページ送り
- **お気に入り** — ランキングから★で追加、一覧から管理・削除
- **既読位置記録** — リーダーでページ読み込み成功時に自動で進捗を保存

## 詳細ドキュメント

- [開発ガイド](docs/development.md) — ローカルでの開発・ビルド方法
- [API リファレンス](docs/api.md) — REST API の仕様
- [アーキテクチャ](docs/architecture.md) — バックエンド / フロントエンドの構成
