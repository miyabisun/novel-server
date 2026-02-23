# アーキテクチャ

## ディレクトリ構造

```
novel-server/
├── Cargo.toml                  # Rust プロジェクト定義
├── src/                        # バックエンド（Rust）
│   ├── main.rs                 # エントリポイント（axum + tokio::main）
│   ├── config.rs               # 環境変数の読み込み
│   ├── error.rs                # AppError enum + IntoResponse
│   ├── state.rs                # AppState（db, cache, config, http）
│   ├── db.rs                   # SQLite 初期化（rusqlite）
│   ├── cache.rs                # インメモリキャッシュ（HashMap + TTL）
│   ├── sanitize.rs             # HTML サニタイズ（ammonia）
│   ├── spa.rs                  # SPA 用 index.html 配信
│   ├── sync.rs                 # お気に入りバックグラウンド同期
│   ├── modules/
│   │   ├── mod.rs              # ModuleType enum + dispatch
│   │   ├── syosetu.rs          # なろう系共通ユーティリティ
│   │   ├── narou.rs            # なろうスクレイピング
│   │   ├── kakuyomu.rs         # カクヨムスクレイピング
│   │   └── nocturne.rs         # ノクターンスクレイピング
│   └── routes/
│       ├── mod.rs              # ルーター組み立て
│       ├── detail.rs           # 小説詳細 API
│       ├── favorites.rs        # お気に入り CRUD
│       ├── ranking.rs          # ランキング API
│       ├── search.rs           # 検索 API
│       ├── toc.rs              # 目次 API
│       └── pages.rs            # 小説本文 API
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
| Web フレームワーク | axum + tokio |
| データベース | rusqlite (bundled SQLite) |
| HTML パース | scraper |
| HTML サニタイズ | ammonia |
| HTTP クライアント | reqwest |
| 日時処理 | chrono |
| 環境変数 | dotenvy |
| シリアライズ | serde + serde_json |
| ログ | tracing + tracing-subscriber |
| エラー型 | thiserror |

### 設計方針

- **enum dispatch**: `ModuleType { Narou, Nocturne, Kakuyomu }` で 3 モジュールを切り替え。trait objects より単純で型安全。
- **DB**: `Arc<Mutex<rusqlite::Connection>>` — SQLite は高速なので非同期プール不要。Mutex guard は `{}` ブロック内で完結させ `.await` を跨がない。
- **Cache**: `Arc<Mutex<HashMap<String, CacheEntry>>>` — 最大 10k 件、serde_json::Value で異種データ格納。
- **Background sync**: `tokio::spawn` + `tokio::time::interval`（narou/nocturne 10 分）、`tokio::time::sleep` チェーン（kakuyomu 動的間隔）。

### リクエスト処理の流れ

```
リクエスト
  → tracing logger
  → [basePath でルーティング]
  → ルートハンドラ
  → レスポンス
```

### スクレイピングモジュール

`src/modules/` 内の各モジュールは `ModuleType` の enum dispatch により以下のメソッドを提供:

- `fetch_ranking_list(limit, period)` — ランキングデータを取得してジャンル別にグループ化（period: `daily` / `weekly` / `monthly` / `quarter` / `yearly`）
- `fetch_search(word)` — キーワードで小説を検索（最大 20 件、評価順）
- `fetch_toc(id)` — 小説の目次（全エピソードのタイトルと番号）を取得
- `fetch_detail(id)` — 小説のタイトル・あらすじ・総ページ数を取得
- `fetch_page(id, num)` — 小説の本文 HTML を取得
- `fetch_data(ids)` — 複数小説のメタデータを一括取得（同期用）
- `fetch_datum(id)` — 単一小説のメタデータを取得（同期用）

### HTML サニタイズ

`src/sanitize.rs` では ammonia を使用し、スクレイピングモジュールが返す本文 HTML をサニタイズしてからクライアントに返しています。

#### 許可タグ

```
p, br, hr, div, span,
h1, h2, h3, h4, h5, h6,
ruby, rt, rp, rb,
em, strong, b, i, u, s, sub, sup
```

- **許可タグ**: 全属性を削除した上でタグを保持
- **非許可タグ**: タグを削除しテキストコンテンツのみ保持

#### 全属性を削除する理由

スクレイピング元の HTML には `style`, `class`, `id` のような無害な属性に加え、`onclick`, `onerror` 等のイベントハンドラ属性が含まれる可能性がある。属性を個別に許可/拒否する方式は漏れのリスクがあるため、全属性を一律削除している。

### キャッシュ戦略

インメモリの HashMap ベースキャッシュを使用しています（`src/cache.rs`）。

| 対象 | TTL | 説明 |
|------|-----|------|
| ランキング | 3 時間 | 各サイトのランキングは頻繁には更新されない |
| 検索結果 | 1 時間 | 新作投稿を早めに反映するため短めの TTL |
| 小説詳細 | 24 時間 | タイトル・あらすじは基本的に変わらない |
| ページ本文 | 24 時間 | 小説の本文は基本的に変わらない |
| 目次 | なし | リアルタイム性を重視（最新の話数を即時反映） |

キャッシュの強制更新は各エンドポイントの `PATCH` メソッドで行えます。

エラーレスポンスはキャッシュしない設計。`cache.set()` は成功時のみ呼び出されるため、一時的な障害が解消すれば次のリクエストで正常データを取得・キャッシュできる。

### エラーハンドリング

#### リトライの責務分離

| 層 | リトライ | 理由 |
|----|---------|------|
| ルートハンドラ（`routes/*.rs`） | 3 回 + 線形バックオフ（500ms × 試行回数） | ユーザーリクエストに対する応答品質を保証 |
| スクレイピングモジュール（`modules/*.rs`） | なし | 単純な fetch に徹し、リトライ判断は呼び出し元に委ねる |
| バックグラウンド同期（`sync.rs`） | なし（ループ継続で暗黙的リトライ） | 次の周期で自動的に再試行される |

#### サイト構造変更への対応

scraper セレクタが空を返した場合、各モジュールは `None` または空配列を返す。現状は明示的な検知・通知機構はなく、サーバーログで確認する。

### バックグラウンド同期

`src/sync.rs` がお気に入りに登録された小説のメタデータ（タイトル・ページ数・更新日時）を定期的に同期します。

| サイト | 方式 | 間隔 |
|--------|------|------|
| narou / nocturne | `fetch_data` で全件一括取得 | 10 分 |
| kakuyomu | `fetch_datum` で 1 件ずつラウンドロビン | 1 時間で全件を巡回 |

#### narou / nocturne（tokio::time::interval）

なろう API は複数 ID を一括取得できるため、処理時間が短く安定している。固定間隔の interval で十分。

#### kakuyomu（tokio::time::sleep チェーン）

HTML スクレイピングで 1 件ずつ取得するため、お気に入り件数に応じて間隔を動的に計算する。`sleep(3_600_000ms / count)` で 1 時間かけて全件を均等に巡回する。

#### スケーリング特性（kakuyomu）

間隔は `3,600,000ms ÷ 件数` で計算される:

| 件数 | 1 件あたりの間隔 |
|------|-----------------|
| 10 | 6 分 |
| 100 | 36 秒 |
| 500 | 7.2 秒 |

#### エラー時の挙動

- **narou / nocturne**: 例外をログして継続。interval は次の周期で自動再実行
- **kakuyomu**: 例外をログし、60 秒待機後に sleep で再スケジュール

### データベース

rusqlite (bundled SQLite) を使用。起動時に `db.rs` が `CREATE TABLE IF NOT EXISTS` でスキーマを自動作成します。

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
