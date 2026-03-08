# スクレイピングモジュール追加ガイド

新しい小説サイトのスクレイピングモジュールを追加する手順。

## 必須関数

`src/modules/module_type.nim` の `ModuleType` enum dispatch に準拠する 7 関数を実装する。

| 関数 | 用途 | 呼び出し元 |
|------|------|-----------|
| `fetchRankingList(client, limit, period)` | ランキング取得（ジャンル別グループ化） | `routes/ranking.nim` |
| `fetchDetail(client, id)` | 小説メタデータ（タイトル・あらすじ・ページ数） | `routes/detail.nim` |
| `fetchPage(client, id, num)` | 小説本文 HTML | `routes/pages.nim` |
| `fetchSearch(client, word)` | キーワード検索 | `routes/search.nim` |
| `fetchToc(client, id)` | 目次（全エピソード一覧） | `routes/toc.nim` |
| `fetchData(client, ids)` | 複数小説のメタデータ一括取得 | `sync.nim`（一括同期） |
| `fetchDatum(client, id)` | 単一小説のメタデータ取得 | `sync.nim`（ラウンドロビン同期） |

`fetchData` / `fetchDatum` の戻り値には最低限 `title`, `pages` (配列), `novelupdated_at` を含める。

## 同期パターンの選択

対象サイトの API 特性に応じて同期方式が異なる。

### 一括取得 API がある場合（narou / nocturne 型）

- `fetchData(client, ids)` で複数 ID を一括取得
- `sync.nim` の `startSyosetuSync` を参考に `sleepAsync` で固定間隔実行

### 1 件ずつしか取得できない場合（kakuyomu 型）

- `fetchData(client, ids)` は `fetchDatum` を順次呼び出しで実装（レート制限対策で sleepAsync を挟む）
- `sync.nim` の `startKakuyomuSync` を参考に `sleepAsync` チェーンで動的間隔実行

## 追加手順

1. `src/modules/<site>.nim` を作成し、7 関数を実装
2. `src/modules/module_type.nim` の `ModuleType` enum にバリアントを追加
3. `ModuleType` の各メソッド（`resolve`, `asStr`, `fetch*`）に `case` アームを追加
4. HTML パース結果が `sanitize.clean()` を通過するか確認
5. ランキング取得が正しいジャンル別グループ構造（`JsonNode` オブジェクト）を返すか確認
6. `sync.nim` の `startSync` に同期ループを追加
7. フロントエンド側のサイト種別リストに追加（該当する場合）

## 既存モジュールの参考構造

- **narou / nocturne**: `syosetu.nim` の共通ユーティリティ（`fetchApi`, `mapItem`, `parsePage`, `parseToc`）を共有。なろう系列サイトなら同様に共有可能
- **kakuyomu**: HTML スクレイピング + `#__NEXT_DATA__` の Apollo State パース。スクレイピング主体のサイトはこちらを参考に
