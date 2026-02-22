# ドキュメント追加提案

youtube-sub-feed の開発・レビューを通じて得た知見から、novel-server のドキュメントに追加すると有益と思われる項目をまとめる。

---

## 1. テスト戦略ドキュメント (`testing.md`)

**背景**: youtube-sub-feed では TDD で 127 テスト（17 ファイル）を構築した。その過程で「モジュールスコープの副作用（DB 接続）があるとテストが書けない」問題に直面し、DI（依存性注入）パターンで解決した。novel-server にはテストが存在しない。

**追加内容**:

- **テスト方針**: どの層をテストするか（純粋関数 > ビジネスロジック > ルートハンドラ）
- **DI パターンの適用ガイド**: `favorite-sync.ts` や `routes/*.ts` が `db` をモジュールスコープでインポートしているため、テスト時に in-memory SQLite に差し替えられない。youtube-sub-feed で採用した「Core 関数 + 薄いラッパー」パターンの適用方法

```typescript
// Before: テスト困難
import { db } from '../db/index.js'
export function syncFavorites() { db.select(...) }

// After: テスト容易
export function syncFavoritesCore(deps: { db: Database }) { deps.db.select(...) }
export function syncFavorites() { return syncFavoritesCore({ db }) }
```

- **テスト対象の優先順位**:
  1. `modules/*.ts` — スクレイピング結果のパース（HTML 構造変更への耐性確認）
  2. `routes/favorites.ts` — CRUD + progress 更新のビジネスロジック
  3. `lib/init.ts` — スキーマ冪等性・デフォルト値
  4. `routes/pages.ts` — HTML サニタイズの許可リスト検証

---

## 2. スクレイピングモジュール追加ガイド (`guides/add-module.md`)

**背景**: `modules/` に narou, kakuyomu, nocturne の 3 モジュールがあるが、新サイト追加時の手順が文書化されていない。youtube-sub-feed で YouTube API クライアントを構築した際、インターフェースの統一が後工程の品質を大きく左右した。

**追加内容**:

- **必須インターフェース**: 各モジュールが実装すべき 5 関数（`fetchRankingList`, `fetchDetail`, `fetchPage`, `fetchData`, `fetchDatum`）の型定義と期待する戻り値
- **同期パターンの選択基準**: 一括取得 API があるか（narou 型）、1 件ずつしか取得できないか（kakuyomu 型）で `favorite-sync.ts` への登録方法が変わる
- **チェックリスト**: HTML パース → サニタイズ通過確認 → ランキング取得 → お気に入り同期 → フロントエンド表示

---

## 3. バックグラウンド同期の設計判断 (`architecture.md` への追記)

**背景**: youtube-sub-feed のポーリングシステム（通常 30 分 / 高頻度 10 分の 2 ループ並行）を設計する際、novel-server の `favorite-sync.ts` を直接参考にした。しかし「なぜ setInterval と setTimeout を使い分けているのか」「なぜ kakuyomu だけラウンドロビンなのか」が既存ドキュメントから読み取れなかった。

**追記内容**:

- **setInterval vs setTimeout チェーン**: narou/nocturne は全件一括取得（処理時間が短い）ため setInterval で十分。kakuyomu は 1 件ずつ取得するためチャンネル数に応じて間隔を動的計算する必要があり、setTimeout チェーンを採用
- **スケーリング特性**: kakuyomu のお気に入りが増えた場合の間隔変化（100 件 → 36 秒/件、500 件 → 7.2 秒/件）
- **エラー時の挙動**: fetch 失敗時にループが停止するか継続するか

---

## 4. エラーハンドリング方針

**背景**: youtube-sub-feed では YouTube API のクォータ超過、トークン期限切れ、ネットワークエラーなど多様なエラーパターンに対処した。novel-server のスクレイピングモジュールにも同様の問題（サイト構造変更、レート制限、タイムアウト）があるが、方針が文書化されていない。

**追加内容（`architecture.md` または独立ファイル）**:

- **リトライ方針**: `routes/detail.ts` と `routes/pages.ts` で 3 回リトライ + 線形バックオフを実装済みだが、`modules/*.ts` 内の fetch にはリトライがない。どこでリトライすべきか（ルート層 vs モジュール層）の方針
- **サイト構造変更への対応**: cheerio セレクタが空を返した場合の検知方法と通知手段
- **キャッシュとエラーの関係**: エラーレスポンスをキャッシュしない現在の設計が意図的かどうか

---

## 5. HTML サニタイズ仕様の詳細化 (`architecture.md` への追記)

**背景**: `architecture.md` にサニタイズの概要はあるが、許可タグの完全なリストが記載されていない。youtube-sub-feed のレビューで「入力検証はシステム境界で行う」原則を改めて確認した。novel-server ではスクレイピング結果（外部入力）を HTML として返すため、許可リストの明示は重要。

**追記内容**:

- 許可タグの完全なリスト（現在 `pages.ts` のコード内にのみ存在）
- 全属性を削除する理由（`onclick`, `onerror` などの XSS ベクター排除）
- `HTMLRewriter` を選択した理由（cheerio との比較は記載済みだが、セキュリティ面の考慮も追記）

---

## 6. デプロイ・運用ドキュメントの拡充 (`development.md` への追記)

**背景**: youtube-sub-feed では `deploy.md` に nginx リバースプロキシ設定例と volume マウントの注意点を記載した。novel-server の `development.md` は Docker ビルド・起動のみで、BASE_PATH 設定時のリバースプロキシ構成例がない。

**追記内容**:

- nginx / Caddy でのリバースプロキシ設定例（BASE_PATH あり/なし）
- SQLite WAL モードでの volume マウント注意点（`*.db-wal`, `*.db-shm` が同一ディレクトリに必要）
- バックアップ手順（SQLite の `.backup` コマンドまたはファイルコピー）

---

## 優先度

| 優先度 | 項目 | 理由 |
|--------|------|------|
| 高 | 1. テスト戦略 | テスト 0 件は最大のリスク。スクレイピング対象サイトの構造変更時に回帰検知できない |
| 高 | 3. 同期の設計判断 | 新プロジェクトで参考にする際に最も情報が不足していた箇所 |
| 中 | 2. モジュール追加ガイド | 新サイト追加時に都度コード読解が必要な状態 |
| 中 | 4. エラーハンドリング方針 | 暗黙知のまま放置するとサイト障害時に対応が遅れる |
| 低 | 5. サニタイズ仕様 | セキュリティ上重要だが現在の実装で動作しているため緊急度は低い |
| 低 | 6. デプロイ拡充 | 現在の Docker 手順で運用できているため |
