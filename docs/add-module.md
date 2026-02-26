# スクレイピングモジュール追加ガイド

新しい小説サイトのスクレイピングモジュールを追加する手順。

## 必須関数

`src/modules/mod.rs` の `ModuleType` enum dispatch に準拠する 7 関数を実装する。

| 関数 | 用途 | 呼び出し元 |
|------|------|-----------|
| `fetch_ranking_list(client, limit, period)` | ランキング取得（ジャンル別グループ化） | `routes/ranking.rs` |
| `fetch_detail(client, id)` | 小説メタデータ（タイトル・あらすじ・ページ数） | `routes/detail.rs` |
| `fetch_page(client, id, num)` | 小説本文 HTML | `routes/pages.rs` |
| `fetch_search(client, word)` | キーワード検索 | `routes/search.rs` |
| `fetch_toc(client, id)` | 目次（全エピソード一覧） | `routes/toc.rs` |
| `fetch_data(client, ids)` | 複数小説のメタデータ一括取得 | `sync.rs`（一括同期） |
| `fetch_datum(client, id)` | 単一小説のメタデータ取得 | `sync.rs`（ラウンドロビン同期） |

`fetch_data` / `fetch_datum` の戻り値には最低限 `title`, `page`, `novelupdated_at` を含める。

## 同期パターンの選択

対象サイトの API 特性に応じて同期方式が異なる。

### 一括取得 API がある場合（narou / nocturne 型）

- `fetch_data(client, ids)` で複数 ID を一括取得
- `sync.rs` の `start_syosetu_sync` を参考に `tokio::time::interval` で固定間隔実行

### 1 件ずつしか取得できない場合（kakuyomu 型）

- `fetch_data(client, ids)` は `fetch_datum` を順次呼び出しで実装（レート制限対策で sleep を挟む）
- `sync.rs` の `start_kakuyomu_sync` を参考に `tokio::time::sleep` チェーンで動的間隔実行

## 追加手順

1. `src/modules/<site>.rs` を作成し、7 関数を実装
2. `src/modules/mod.rs` の `ModuleType` enum にバリアントを追加
3. `ModuleType` の各メソッド（`resolve`, `as_str`, `fetch_*`）に `match` アームを追加
4. HTML パース結果が `sanitize::clean()` を通過するか確認
5. ランキング取得が正しいジャンル別グループ構造（`HashMap<String, Vec<Value>>`）を返すか確認
6. `sync.rs` の `start_sync` に同期ループを追加
7. フロントエンド側のサイト種別リストに追加（該当する場合）

## 既存モジュールの参考構造

- **narou / nocturne**: `syosetu.rs` の共通ユーティリティ（`fetch_api`, `map_item`, `parse_page`, `parse_toc`）を共有。なろう系列サイトなら同様に共有可能
- **kakuyomu**: HTML スクレイピング + `#__NEXT_DATA__` の Apollo State パース。スクレイピング主体のサイトはこちらを参考に
