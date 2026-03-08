# テスト戦略

## テストの実行

```bash
# 単体テスト
nimble test

# インテグレーションテスト（サーバービルド + 起動 + API検証）
bash test/integration.sh

# インテグレーションテスト（ライブAPI含む）
LIVE_TESTS=1 bash test/integration.sh

# 手動テスト（サーバー起動後にブラウザで確認）
nim c -r src/main.nim
```

## 方針

純粋関数 > ビジネスロジック の優先順位で、外部依存を持たないコードから着手する。

## テスト対象

### 単体テスト

#### `test/t_syosetu.nim` — API レスポンス処理・パース

- `mapItem()`: ncode→id変換（小文字化）、title strip、general_all_no→page変換、null/数値型の堅牢性
- `processApiResponse()`: メタデータスキップ、非オブジェクトフィルタリング
- `buildPages()`: ページリスト生成
- `parsePage()`: セレクタによるHTML抽出、複数マッチの結合、空/不在時のnone返却
- `parseToc()`: タイトル・エピソード抽出、titleフォールバック、ページネーション解析
- `OfRanking` / `OfDatum` / `OfDetail` 定数のフォーマット検証（ハイフン区切り、カンマ不使用）

#### `test/t_sanitize.nim` — HTML サニタイズ

- 許可タグの保持、自己閉じタグ
- XSS ベクター除去（`<script>`, `<img onerror>`, `<style>`, `<noscript>`, `<template>`, `<title>`）
- イベントハンドラ属性除去、全属性ストリップ
- HTMLコメント除去
- 非許可タグのテキスト保持
- 許可タグと非許可タグの混在処理

### インテグレーションテスト

#### `test/integration.sh` — API エンドポイント検証

- SPA配信、静的ファイル
- Favorites CRUD（作成・取得・更新・削除）
- エラーハンドリング（バリデーション、不正パラメータ）
- ランキングAPI（`LIVE_TESTS=1` 時のみ、外部API依存）
