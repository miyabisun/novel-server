# テスト戦略

## テストの実行

```bash
bun test            # 全テスト実行
bun test src/lib    # ディレクトリ指定
```

テストファイルはソースと同じディレクトリに `*.test.ts` として配置する。

## 方針

純粋関数 > ビジネスロジック > ルートハンドラ の優先順位で、外部依存を持たないコードから着手する。

## DI パターンの適用

現在のコードはモジュールスコープで `db` をインポートしているため、テスト時に in-memory SQLite に差し替えられない。テスト可能にするには **Core 関数 + 薄いラッパー** パターンを適用する。

```typescript
// Before: テスト困難（db がモジュールスコープで固定）
import { db } from '../db/index.js'
export function syncFavorites() { db.select(...) }

// After: テスト容易（db を外部から注入可能）
// ※ 疑似コード。実際の型は BunSQLiteDatabase<typeof schema>
export function syncFavoritesCore(deps: { db: DbClient }) { deps.db.select(...) }
export function syncFavorites() { return syncFavoritesCore({ db }) }
```

テスト側では in-memory SQLite で Drizzle インスタンスを生成し、Core 関数に渡す。

## テスト対象の優先順位

### 1. `modules/*.ts` — パース処理

スクレイピング結果のパース（HTML → 構造化データ）はサイトの HTML 構造変更に対する回帰テストとして最も価値が高い。

- `syosetu.ts`: `mapItem()`, `parsePage()`, `createFetchApi()` のレスポンスマッピング
- `kakuyomu.ts`: Apollo State のパース、ページ HTML 抽出
- `narou.ts` / `nocturne.ts`: `syosetu.ts` をベースとした差分のみ確認

### 2. `routes/favorites.ts` — CRUD + 進捗更新

upsert ロジック、ソート順（`novelupdated_at desc nulls last`）、進捗更新時の未登録チェック。DB 操作が中心のため DI パターン適用後にテスト。

### 3. `lib/init.ts` — スキーマ冪等性

`CREATE TABLE IF NOT EXISTS` の冪等性（2 回実行してもエラーにならないこと）と、`read` カラムのデフォルト値（0）の検証。

### 4. `lib/sanitize.ts` — HTML サニタイズ

許可タグリストに基づく `sanitizeHtml()` の動作確認。XSS ベクター（`<script>`, `<img onerror>` 等）が除去されることの検証。
