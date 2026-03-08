# アーキテクチャ

## ディレクトリ構造

```
novel-server/
├── novel_server.nimble         # Nim パッケージ定義
├── nim.cfg                     # コンパイラ設定
├── src/                        # バックエンド（Nim）
│   ├── main.nim                # エントリポイント（Jester + asyncdispatch）
│   ├── config.nim              # 環境変数の読み込み
│   ├── error.nim               # AppError 例外型
│   ├── state.nim               # AppState（db, cache, config, http）
│   ├── db.nim                  # SQLite 初期化（db_sqlite）
│   ├── cache.nim               # インメモリキャッシュ（Table + TTL）
│   ├── sanitize.nim            # HTML サニタイズ（allowlist 方式）
│   ├── spa.nim                 # SPA 用 index.html 配信
│   ├── sync.nim                # お気に入りバックグラウンド同期
│   ├── modules/
│   │   ├── module_type.nim     # ModuleType enum + dispatch
│   │   ├── syosetu.nim         # なろう/ノクターン共通処理
│   │   └── kakuyomu.nim        # カクヨムスクレイピング
│   └── routes/
│       ├── router.nim          # Jester ルーター組み立て
│       ├── detail.nim          # 小説詳細 API
│       ├── favorites.nim       # お気に入り CRUD
│       ├── ranking.nim         # ランキング API
│       ├── search.nim          # 検索 API
│       ├── toc.nim             # 目次 API
│       └── pages.nim           # 小説本文 API
├── client/                     # フロントエンド（Svelte 5）
│   ├── src/
│   │   ├── App.svelte          # ルートコンポーネント
│   │   ├── main.js             # エントリポイント
│   │   ├── lib/
│   │   │   ├── config.js       # API パス設定
│   │   │   ├── decode.js       # HTML エンティティデコード
│   │   │   ├── fetcher.js      # fetch ラッパー
│   │   │   ├── router.svelte.js # SPA ルーター
│   │   │   └── components/
│   │   │       ├── ConfirmModal.svelte
│   │   │       ├── Header.svelte
│   │   │       └── NovelDetailModal.svelte
│   │   └── pages/
│   │       ├── Ranking.svelte          # ランキング一覧
│   │       ├── Reader.svelte           # 小説リーダー
│   │       ├── TableOfContents.svelte  # 目次
│   │       └── Favorites.svelte        # お気に入り一覧
│   └── build/                  # ビルド出力（git 管理外）
├── Dockerfile                  # マルチステージビルド
└── .env                        # 環境変数（DATABASE_PATH, PORT, BASE_PATH）
```

## バックエンド

### 技術スタック

| 用途 | ライブラリ |
|------|-----------|
| Web フレームワーク | Jester + asyncdispatch |
| データベース | db_sqlite (stdlib) |
| HTML パース | htmlparser (stdlib) + nimquery |
| HTML サニタイズ | カスタム実装（allowlist 方式） |
| HTTP クライアント | httpclient (stdlib) |
| 日時処理 | times (stdlib) |
| 環境変数 | カスタム .env ローダー |
| シリアライズ | json (stdlib) |
| ログ | logging (stdlib) |
| 正規表現 | re (stdlib) |

### 設計方針

- **enum dispatch**: `ModuleType { Narou, Nocturne, Kakuyomu }` で 3 モジュールを切り替え。
- **DB**: `DbConn` + `Lock` — SQLite は高速なので非同期プール不要。Lock で排他制御し acquire/release をブロック内で完結。
- **Cache**: `Table[string, CacheEntry]` + `Lock` — 最大 10k 件、JsonNode で異種データ格納。
- **Background sync**: `asyncCheck` + `sleepAsync`（narou/nocturne 10 分固定間隔）、sleep チェーン（kakuyomu 動的間隔）。

### リクエスト処理の流れ

```
リクエスト
  → Jester router
  → ルートハンドラ (→ エラーハンドリング)
  → JSON レスポンス
```

### スクレイピングモジュール

`src/modules/` 内の各モジュールは `ModuleType` の enum dispatch により以下のメソッドを提供:

- `fetchRankingList(client, limit, period)` — ランキングデータを取得してジャンル別にグループ化（period: `daily` / `weekly` / `monthly` / `quarter` / `yearly`）
- `fetchSearch(client, word)` — キーワードで小説を検索（最大 20 件、評価順）
- `fetchToc(client, id)` — 小説の目次（全エピソードのタイトルと番号）を取得
- `fetchDetail(client, id)` — 小説のタイトル・あらすじ・総ページ数を取得
- `fetchPage(client, id, num)` — 小説の本文 HTML を取得
- `fetchData(client, ids)` — 複数小説のメタデータを一括取得（同期用）
- `fetchDatum(client, id)` — 単一小説のメタデータを取得（同期用）

### HTML サニタイズ

`src/sanitize.nim` ではカスタム実装の allowlist 方式でサニタイズしています。

#### 許可タグ

```
p, br, hr, div, span,
h1, h2, h3, h4, h5, h6,
ruby, rt, rp, rb,
em, strong, b, i, u, s, sub, sup
```

- **許可タグ**: 全属性を削除した上でタグを保持
- **非許可タグ**: タグを削除しテキストコンテンツのみ保持
- **コンテンツ削除タグ**: script, style, title, noscript, template はコンテンツごと削除

#### 全属性を削除する理由

スクレイピング元の HTML には `style`, `class`, `id` のような無害な属性に加え、`onclick`, `onerror` 等のイベントハンドラ属性が含まれる可能性がある。属性を個別に許可/拒否する方式は漏れのリスクがあるため、全属性を一律削除している。

### キャッシュ戦略

インメモリの Table ベースキャッシュを使用しています（`src/cache.nim`）。

| 対象 | TTL | 説明 |
|------|-----|------|
| ランキング | 3 時間 | 各サイトのランキングは頻繁には更新されない |
| 検索結果 | 1 時間 | 新作投稿を早めに反映するため短めの TTL |
| 小説詳細 | 24 時間 | タイトル・あらすじは基本的に変わらない |
| ページ本文 | 24 時間 | 小説の本文は基本的に変わらない |
| 目次 | なし | リアルタイム性を重視（最新の話数を即時反映） |

キャッシュの強制更新は各エンドポイントの `PATCH` メソッドで行えます。

エラーレスポンスはキャッシュしない設計。`cache.put()` は成功時のみ呼び出されるため、一時的な障害が解消すれば次のリクエストで正常データを取得・キャッシュできる。

### エラーハンドリング

#### リトライの責務分離

| 層 | リトライ | 理由 |
|----|---------|------|
| ルートハンドラ（`routes/*.nim`） | 3 回 + 線形バックオフ（500ms × 試行回数） | ユーザーリクエストに対する応答品質を保証 |
| スクレイピングモジュール（`modules/*.nim`） | なし | 単純な fetch に徹し、リトライ判断は呼び出し元に委ねる |
| バックグラウンド同期（`sync.nim`） | なし（ループ継続で暗黙的リトライ） | 次の周期で自動的に再試行される |

#### サイト構造変更への対応

nimquery セレクタが空を返した場合、各モジュールは `none` または空配列を返す。現状は明示的な検知・通知機構はなく、サーバーログで確認する。

### バックグラウンド同期

`src/sync.nim` がお気に入りに登録された小説のメタデータ（タイトル・ページ数・更新日時）を定期的に同期します。

| サイト | 方式 | 間隔 |
|--------|------|------|
| narou / nocturne | `fetchData` で全件一括取得 | 10 分 |
| kakuyomu | `fetchDatum` で 1 件ずつラウンドロビン | 1 時間で全件を巡回 |

#### narou / nocturne（sleepAsync interval）

なろう API は複数 ID を一括取得できるため、処理時間が短く安定している。固定間隔の sleepAsync ループで十分。

#### kakuyomu（sleepAsync チェーン）

HTML スクレイピングで 1 件ずつ取得するため、お気に入り件数に応じて間隔を動的に計算する。`sleepAsync(3_600_000ms / count)` で 1 時間かけて全件を均等に巡回する。

#### スケーリング特性（kakuyomu）

間隔は `3,600,000ms ÷ 件数` で計算される:

| 件数 | 1 件あたりの間隔 |
|------|-----------------|
| 10 | 6 分 |
| 100 | 36 秒 |
| 500 | 7.2 秒 |

#### エラー時の挙動

- **narou / nocturne**: 例外をログして継続。次の周期で自動再実行
- **kakuyomu**: 例外をログし、60 秒待機後に再スケジュール

### データベース

db_sqlite (Nim stdlib) を使用。起動時に `db.nim` が `CREATE TABLE IF NOT EXISTS` でスキーマを自動作成します。

DB パスは環境変数 `DATABASE_PATH` で指定します（デフォルト: `/data/novel.db`）。

**favorites** — お気に入り管理

| カラム | 型 | 説明 |
|--------|-----|------|
| `type` | TEXT (PK) | サイト種別 |
| `id` | TEXT (PK) | 小説 ID |
| `title` | TEXT | タイトル |
| `novelupdated_at` | TEXT? | 小説の更新日時 |
| `page` | INTEGER | 総ページ数 |
| `read` | INTEGER | 既読ページ番号 |

## フロントエンド

### デザインシステム

`global.sass` の `:root` に CSS custom properties としてデザイントークンを定義しています。[LiftKit](https://liftkit.happykit.dev/) の設計思想をベースに、黄金比（φ ≈ 1.618）でスケーリングしています。

| カテゴリ | プレフィックス | 例 |
|----------|----------------|-----|
| Spacing | `--sp-*` | `--sp-1: 4px` 〜 `--sp-6: 42px` |
| Font Size | `--fs-*` | `--fs-xs: 0.72rem` 〜 `--fs-xl: 1.62rem` |
| Border Radius | `--radius-*` | `--radius-sm: 4px` 〜 `--radius-lg: 10px` |
| Color (surface) | `--c-bg`, `--c-surface` | 背景・カード面 |
| Color (text) | `--c-text-*` | `--c-text`（主）〜 `--c-text-faint`（最淡） |
| Color (semantic) | `--c-accent-*`, `--c-danger-*`, `--c-fav-*` | アクセント・危険・お気に入り |

全コンポーネントでハードコードされた値の代わりにこれらのトークンを使用しています。

### ルーティング

自前の SPA ルーター（`router.svelte.js`）を使用。パターンマッチングでルートインデックスを決定します。

| Index | パス | ページ |
|-------|------|--------|
| 0 | `/` | Favorites |
| 1 | `/ranking/:type` | Ranking |
| 2 | `/novel/:type/:id/toc` | TableOfContents |
| 3 | `/novel/:type/:id/:num` | Reader |

### 状態管理

Svelte 5 の `$state` ルーンを使用。グローバルな状態:

- **router** (`router.svelte.js`) — 現在のルートインデックスとパラメータ

### API 通信

`fetcher.js` が fetch のラッパーとして機能します。

### BASE_PATH 対応

リバースプロキシ配下でサブパス運用する場合:

1. サーバー起動時に `index.html` の `<base>` タグと `window.__BASE_PATH__` を書き換え
2. フロントエンドは `getBasePath()` でプレフィックスを取得し、リンクと API パスに付与
