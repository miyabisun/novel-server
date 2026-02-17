# novel-server

なろう・カクヨム・ノクターンに対応した小説ランキングビューア＆リーダー。
お気に入り管理と既読位置の自動記録機能付き。

## 技術スタック

- **バックエンド**: Hono + TypeScript + Prisma (SQLite)
- **フロントエンド**: Svelte 5 + Vite + Sass

## セットアップ

以下のコマンドで依存関係のインストール、`.env` の生成、データベースの初期化をまとめて行います:

```bash
./scripts/install.sh
```

既にセットアップ済みの環境で再実行しても安全です。

### 手動セットアップ

個別に実行する場合:

```bash
npm install                # サーバー側の依存関係
cd client && npm install   # クライアント側の依存関係
npm run init               # .env を生成
npm run db:push            # Prisma スキーマを DB に同期
```

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

## 主な機能

- **ランキング閲覧** — なろう / カクヨム / ノクターンのランキングを期間選択で表示、あらすじモーダル付き
- **小説リーダー** — キーボード（矢印キー）対応のページ送り
- **お気に入り** — ランキングから★で追加、一覧から管理・削除
- **既読位置記録** — リーダーでページ読み込み成功時に自動で進捗を保存

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `./scripts/install.sh` | 初期セットアップ一括実行 |
| `npm run dev` | 開発サーバー起動（vite watch + node） |
| `npm run build` | フロントエンドビルド |
| `npm start` | 本番サーバー起動 |
| `npm run db:push` | Prisma スキーマを DB に同期 |
| `npm run init` | `.env` を生成 |

## リバースプロキシ配下での運用

`BASE_PATH` を設定するとサブパスで動作します:

```env
BASE_PATH=/novels
```

## ドキュメント

- [API リファレンス](docs/api.md)
- [アーキテクチャ](docs/architecture.md)
