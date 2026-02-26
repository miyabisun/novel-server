# ドキュメント追加提案

youtube-sub-feed の開発・レビューを通じて得た知見から、novel-server のドキュメントに追加すると有益と思われる項目をまとめる。

---

## 1. テストカバレッジの拡充

**現状**: `cargo test` で 11 テスト（スクレイピングモジュールのパース系テスト中心）。Rust では関数の引数で依存を受け取るため、旧 TypeScript 時代の DI 問題は存在しない。

**追加候補**:

- `modules/*.rs` のパーステスト拡充（HTML 構造変更への耐性確認）
- `sanitize.rs` の許可タグ検証（XSS ベクターを含む HTML 入力）
- `cache.rs` の get/set/eviction/sweep 動作
- `db.rs` のスキーマ冪等性（in-memory SQLite で検証可能）

---

## 2. デプロイ・運用ドキュメントの拡充 (`development.md` への追記)

**背景**: `development.md` は Docker ビルド・起動のみで、BASE_PATH 設定時のリバースプロキシ構成例がない。

**追記候補**:

- nginx / Caddy でのリバースプロキシ設定例（BASE_PATH あり/なし）
- SQLite WAL モードでの volume マウント注意点（`*.db-wal`, `*.db-shm` が同一ディレクトリに必要）
- バックアップ手順（SQLite の `.backup` コマンドまたはファイルコピー）

---

## 対応済みの項目

以下は Rust 移行および `architecture.md` の整備により対応済み:

| 項目 | 対応先 |
|------|--------|
| バックグラウンド同期の設計判断 | `architecture.md` — interval vs sleep チェーン、スケーリング特性、エラー時の挙動 |
| エラーハンドリング方針 | `architecture.md` — リトライの責務分離、サイト構造変更への対応 |
| HTML サニタイズ仕様 | `architecture.md` — 許可タグ一覧、全属性削除の理由 |
| スクレイピングモジュール追加ガイド | `add-module.md` — Rust の enum dispatch パターンに基づく手順 |
