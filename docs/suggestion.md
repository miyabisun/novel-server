# ドキュメント追加提案

youtube-sub-feed の開発・レビューを通じて得た知見から、novel-server のドキュメントに追加すると有益と思われる項目をまとめる。

---

## 1. テストカバレッジの拡充

**現状**: `test/integration.sh` でインテグレーションテスト（22 項目）。Nim では関数の引数で依存を受け取るため、旧 TypeScript 時代の DI 問題は存在しない。

**追加候補**:

- `modules/*.nim` のパーステスト拡充（HTML 構造変更への耐性確認）
- `sanitize.nim` の許可タグ検証（XSS ベクターを含む HTML 入力）
- `cache.nim` の get/put/eviction/sweep 動作
- `db.nim` のスキーマ冪等性（in-memory SQLite で検証可能）

---

## 対応済みの項目

以下は `architecture.md` および `development.md` の整備により対応済み:

| 項目 | 対応先 |
|------|--------|
| バックグラウンド同期の設計判断 | `architecture.md` — interval vs sleep チェーン、スケーリング特性、エラー時の挙動 |
| エラーハンドリング方針 | `architecture.md` — リトライの責務分離、サイト構造変更への対応 |
| HTML サニタイズ仕様 | `architecture.md` — 許可タグ一覧、全属性削除の理由 |
| スクレイピングモジュール追加ガイド | `add-module.md` — Nim の enum dispatch パターンに基づく手順 |
| リバースプロキシ構成例 | `development.md` — nginx 設定例（BASE_PATH あり/なし） |
| SQLite WAL volume マウント | `development.md` — ディレクトリ単位マウントの注意点 |
| バックアップ手順 | `development.md` — `.backup` コマンドの使用方法 |
