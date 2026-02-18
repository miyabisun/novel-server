# アーキテクチャ

## ディレクトリ構造

```
novel-server/
├── src/                        # バックエンド（TypeScript）
│   ├── index.ts                # エントリポイント、ミドルウェア設定
│   ├── db/
│   │   ├── index.ts            # Drizzle クライアント（bun:sqlite）
│   │   └── schema.ts           # Drizzle スキーマ定義
│   ├── lib/
│   │   ├── cache.ts            # インメモリキャッシュ（Map + TTL）
│   │   ├── favorite-sync.ts     # お気に入りバックグラウンド同期
│   │   ├── init.ts             # 起動時スキーマ初期化（CREATE IF NOT EXISTS）
│   │   └── spa.ts              # SPA 用 index.html 配信
│   ├── modules/
│   │   ├── index.ts            # モジュール集約
│   │   ├── narou.ts            # なろうスクレイピング
│   │   ├── kakuyomu.ts         # カクヨムスクレイピング
│   │   └── nocturne.ts         # ノクターンスクレイピング
│   └── routes/
│       ├── detail.ts           # 小説詳細 API
│       ├── favorites.ts        # お気に入り CRUD
│       ├── ranking.ts          # ランキング API
│       └── pages.ts            # 小説本文 API
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
│   │       ├── Ranking.svelte  # ランキング一覧
│   │       ├── Reader.svelte   # 小説リーダー
│   │       └── Favorites.svelte # お気に入り一覧
│   └── build/                  # ビルド出力（git 管理外）
├── drizzle.config.ts           # Drizzle Kit 設定
├── Dockerfile                  # マルチステージビルド
├── .env                        # 環境変数（DATABASE_PATH, PORT, BASE_PATH）
└── package.json
```

## バックエンド

### リクエスト処理の流れ

```
リクエスト
  → logger
  → [basePath でルーティング]
  → ルートハンドラ
  → レスポンス
```

### スクレイピングモジュール

`src/modules/` 内の各モジュールは以下のインターフェースを実装しています:

- `fetchRankingList(limit?, period?)` — ランキングデータを取得してジャンル別にグループ化（period: `daily` / `weekly` / `monthly` / `quarter` / `yearly`）
- `fetchDetail(id)` — 小説のタイトル・あらすじ・総ページ数を取得
- `fetchPage(id, num)` — 小説の本文 HTML を取得
- `fetchData(ids)` — 複数小説のメタデータを一括取得（同期用）
- `fetchDatum(id)` — 単一小説のメタデータを取得（同期用）

### HTML サニタイズ

`src/routes/pages.ts` では、スクレイピングモジュールが返す本文 HTML をサニタイズしてからクライアントに返しています。許可タグ以外を除去し、全属性を削除します。

スクレイピングモジュール（`src/modules/`）が HTML パースに cheerio を使用しているのに対し、pages.ts では Bun 組み込みの `HTMLRewriter` を使用しています。cheerio は DOM ツリーを構築するため CSS セレクタによる要素抽出に適していますが、サニタイズのように全要素を順次処理する用途ではストリーミングパーサーの `HTMLRewriter` の方がメモリ効率に優れます。

### キャッシュ戦略

インメモリの Map ベースキャッシュを使用しています（`src/lib/cache.ts`）。

| 対象 | TTL | 説明 |
|------|-----|------|
| ランキング | 3 時間 | 各サイトのランキングは頻繁には更新されない |
| 小説詳細 | 24 時間 | タイトル・あらすじは基本的に変わらない |
| ページ本文 | 24 時間 | 小説の本文は基本的に変わらない |

キャッシュの強制更新は各エンドポイントの `PATCH` メソッドで行えます。

### バックグラウンド同期

`src/lib/favorite-sync.ts` がお気に入りに登録された小説のメタデータ（タイトル・ページ数・更新日時）を定期的に同期します。

| サイト | 方式 | 間隔 |
|--------|------|------|
| narou / nocturne | `fetchData` で全件一括取得 | 10 分 |
| kakuyomu | `fetchDatum` で 1 件ずつラウンドロビン | 1 時間で全件を巡回 |

### データベース

SQLite + Drizzle ORM を使用（`bun:sqlite` ネイティブドライバ）。起動時に `init.ts` が `CREATE TABLE IF NOT EXISTS` でスキーマを自動作成します。

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

### ルーティング

自前の SPA ルーター（`router.svelte.js`）を使用。パターンマッチングでルートインデックスを決定します。

| Index | パス | ページ |
|-------|------|--------|
| 0 | `/` | Favorites |
| 1 | `/ranking/:type` | Ranking |
| 2 | `/novel/:type/:id/:num` | Reader |

### 状態管理

Svelte 5 の `$state` ルーンを使用。グローバルな状態:

- **router** (`router.svelte.js`) — 現在のルートインデックスとパラメータ

### API 通信

`fetcher.js` が fetch のラッパーとして機能します。

### BASE_PATH 対応

リバースプロキシ配下でサブパス運用する場合:

1. サーバー起動時に `index.html` の `<base>` タグと `window.__BASE_PATH__` を書き換え
2. フロントエンドは `getBasePath()` でプレフィックスを取得し、リンクと API パスに付与
