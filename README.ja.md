# novel-server

[English](README.md)

なろう・カクヨム・ノクターンに対応した小説ランキングビューア＆リーダーです。

お気に入りを管理し、既読位置を自動で記録します。

## Quick Start (Docker)

Dockerが必要です。名前付きvolumeで、お気に入りと既読位置をコンテナの再起動後も保持します。

```bash
docker run -p 127.0.0.1:3000:3000 -v novel-data:/data ghcr.io/miyabisun/novel-server:latest
```

ブラウザで `http://localhost:3000` を開く。

## Quick Start (Rust)

Git、Rust、Node.js/npmと、OpenSSL・SQLite向けのビルドツールが必要です。
DockerビルドはRust 1.96を使います。フロントエンドにはNode.js 20.19以上または22.12以上が必要です。

```bash
git clone https://github.com/miyabisun/novel-server.git
cd novel-server
cd client && npm install && npm run build && cd ..
DATABASE_PATH=./novel.db cargo run --release --locked
```

ブラウザで `http://localhost:3000` を開く。

checkout内に`novel.db`を生成します。このファイルとWAL/SHMを保持してください。
別の保存先を使う場合は、書込み可能な親ディレクトリを作ってから`DATABASE_PATH`を設定します。

起動後はサイトとランキング期間を選び、小説を開きます。
★でお気に入りに追加できます。リーダーのページを開くと既読位置を保存します。

## 配備とデータの保存

Dockerの例はlocalhostだけに公開します。native版は全インターフェースで待ち受けます。
アプリ内のログイン機能はなく、識別ヘッダーがない利用者はゲストのデータを共有します。
外部から使う場合はプロキシやネットワーク側でアクセスを制御してください。
Cloudflareの識別ヘッダーを使う場合は、そのプロキシを迂回して接続できない構成にします。

WAL/SHMも保持するため、SQLiteファイルだけでなくデータディレクトリ全体をマウントします。
バックアップと旧DBの復元は[SQLite運用](docs/development.md#sqlite-運用)を参照してください。

プロキシのサブパス配下なら、`BASE_PATH=/novels`などを設定します。
[プロキシの設定例](docs/development.md#リバースプロキシ)も参照してください。

## 環境変数

現行名・必須/任意・既定値・不正値の扱いは
[README.md の Environment Variables](README.md#environment-variables) を正本とします。
`PORT`、`DATABASE_PATH`、`BASE_PATH`、`PUBLIC_BASE_URL`、`NODE_ENV`、`RUST_LOG` を扱います。
サブパス配下で `PUBLIC_BASE_URL` を指定するときは、公開 URL にサブパスも含めてください。
データベースファイルは初回起動時に生成しますが、親ディレクトリは事前に用意してください。

## 主な機能

- **ランキング閲覧** — 期間選択でランキング表示、あらすじモーダル付き、モバイルではスワイプでお気に入り追加・削除
- **小説リーダー** — キーボード（矢印キー）対応のページ送り
- **お気に入り** — ランキングから★で追加、メタデータ（話数・更新日時）を自動同期
- **既読位置記録** — リーダーでページ読み込み成功時に自動で進捗を保存
- **複数ユーザー対応** — お気に入り・既読位置をユーザーごとに分離します。
  Cloudflare Accessが`Cf-Access-Authenticated-User-Email`ヘッダーで利用者を識別します。
  ヘッダーなしの場合はゲストのデータを共有します。

## 詳細ドキュメント

- [開発ガイド](docs/development.md) — ローカルでの開発・ビルド方法
- API リファレンス — サーバー起動後 `/swagger-ui/` で閲覧可能
