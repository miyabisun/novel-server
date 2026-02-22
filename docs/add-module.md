# スクレイピングモジュール追加ガイド

新しい小説サイトのスクレイピングモジュールを追加する手順。

## 必須インターフェース

`src/modules/index.ts` の `NovelModule` インターフェースに準拠する 5 関数を実装する（型定義の詳細はソースを参照）。

| 関数 | 用途 | 呼び出し元 |
|------|------|-----------|
| `fetchRankingList` | ランキング取得（ジャンル別グループ化） | `routes/ranking.ts` |
| `fetchDetail` | 小説メタデータ（タイトル・あらすじ・ページ数） | `routes/detail.ts` |
| `fetchPage` | 小説本文 HTML | `routes/pages.ts` |
| `fetchData` | 複数小説のメタデータ一括取得 | `lib/favorite-sync.ts`（一括同期） |
| `fetchDatum` | 単一小説のメタデータ取得 | `lib/favorite-sync.ts`（ラウンドロビン同期） |

`fetchData` / `fetchDatum` の戻り値には最低限 `title`, `page`, `novelupdated_at` を含める。

## 同期パターンの選択

対象サイトの API 特性に応じて同期方式が異なる。

### 一括取得 API がある場合（narou / nocturne 型）

- `fetchData(ids)` で複数 ID を一括取得
- `favorite-sync.ts` の `startSyosetuSync` に追加（`setInterval` で固定間隔実行）

### 1 件ずつしか取得できない場合（kakuyomu 型）

- `fetchData(ids)` は `fetchDatum` を順次呼び出しで実装（レート制限対策で `sleep` を挟む）
- `favorite-sync.ts` の `startKakuyomuSync` を参考に専用の同期ループを作成（`setTimeout` チェーンで動的間隔）

## 追加手順チェックリスト

1. `src/modules/<site>.ts` を作成し、5 関数を実装
2. `src/modules/index.ts` の modules マップにモジュールを登録
3. HTML パース結果が `routes/pages.ts` のサニタイズを通過するか確認
4. ランキング取得が正しいジャンル別グループ構造を返すか確認
5. `lib/favorite-sync.ts` に同期ループを追加
6. フロントエンド側のサイト種別リストに追加（該当する場合）

## 既存モジュールの参考構造

- **narou / nocturne**: `syosetu.ts` の共通ユーティリティ（`createFetchApi`, `mapItem`, `parsePage`）を共有。なろう系列サイトなら同様に共有可能
- **kakuyomu**: HTML スクレイピング + Apollo State パース。スクレイピング対象サイトはこちらを参考に
