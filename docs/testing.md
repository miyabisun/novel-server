# テスト戦略

## テストの実行

```bash
# インテグレーションテスト（サーバービルド + 起動 + API検証）
bash test/integration.sh

# 手動テスト（サーバー起動後にブラウザで確認）
nim c -r src/main.nim
```

## 方針

純粋関数 > ビジネスロジック の優先順位で、外部依存を持たないコードから着手する。

## テスト対象

### 1. `modules/*.nim` — API レスポンス処理・パース

- `syosetu.nim`: `processApiResponse()` — null/非オブジェクトのフィルタリング、`mapItem()` のフィールド変換
- `syosetu.nim`: `OfRanking` / `OfDatum` / `OfDetail` 定数のフォーマット検証（ハイフン区切り、カンマ不使用）

### 2. `sanitize.nim` — HTML サニタイズ

許可タグリストに基づくサニタイズの動作確認。XSS ベクター（`<script>`, `<img onerror>` 等）が除去されることの検証。
