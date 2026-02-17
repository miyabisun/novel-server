# novel-server

なろう・カクヨム・ノクターンに対応した小説ランキングビューア＆リーダー。
お気に入り管理と既読位置の自動記録機能付き。

## 技術スタック

- **バックエンド**: Hono + TypeScript + Prisma (SQLite)
- **フロントエンド**: Svelte 5 + Vite + Sass
- **認証**: JWT (HS256) + HttpOnly Cookie

## セットアップ

以下のコマンドで依存関係のインストール、`.env` の生成、データベースの初期化をまとめて行います:

```bash
./scripts/install.sh
```

`.env` が存在しない場合のみ対話的に `AUTH_PASSWORD` の入力を求められます（空欄で自動生成）。
既にセットアップ済みの環境で再実行しても安全です。

### 手動セットアップ

個別に実行する場合:

```bash
npm install                # サーバー側の依存関係
cd client && npm install   # クライアント側の依存関係
npm run init               # .env を対話的に生成
npm run db:push            # Prisma スキーマを DB に同期
```

`.env` を手動で作成する場合は `.env.example` を参考にしてください。
`AUTH_USERNAME`、`AUTH_PASSWORD`、`JWT_SECRET` が未設定の場合、サーバーは起動時にエラーで停止します。

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

- **ランキング閲覧** — なろう / カクヨム / ノクターンのランキングを期間選択（日間 / 週間 / 月間 / 四半期 / 年間）で表示
- **小説リーダー** — キーボード（矢印キー）対応のページ送り
- **お気に入り** — ランキングから★で追加、一覧から管理・削除
- **既読位置記録** — リーダーでページ読み込み成功時に自動で進捗を保存
- **認証** — 全 API がログイン必須（シングルユーザー）

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `./scripts/install.sh` | 初期セットアップ一括実行 |
| `npm run dev` | 開発サーバー起動（vite watch + node） |
| `npm run build` | フロントエンドビルド |
| `npm start` | 本番サーバー起動 |
| `npm run db:push` | Prisma スキーマを DB に同期 |
| `npm run init` | `.env` を対話的に生成 |

## リバースプロキシ配下での運用

`BASE_PATH` を設定するとサブパスで動作します:

```env
BASE_PATH=/novels
```

## ドキュメント

- [API リファレンス](docs/api.md)
- [認証の仕組み](docs/authentication.md)
- [アーキテクチャ](docs/architecture.md)
