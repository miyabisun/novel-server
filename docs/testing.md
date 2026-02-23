# テスト戦略

## テストの実行

```bash
cargo test         # 全テスト実行
```

テストはソースファイル内に `#[cfg(test)] mod tests` として配置する。

## 方針

純粋関数 > ビジネスロジック の優先順位で、外部依存を持たないコードから着手する。

## テスト対象

### 1. `modules/*.rs` — API レスポンス処理・パース

- `syosetu.rs`: `process_api_response()` — null/非オブジェクトのフィルタリング、`map_item()` のフィールド変換
- `narou.rs` / `nocturne.rs`: `OF_*` 定数のフォーマット検証（ハイフン区切り、カンマ不使用）

### 2. `sanitize.rs` — HTML サニタイズ

許可タグリストに基づくサニタイズの動作確認。XSS ベクター（`<script>`, `<img onerror>` 等）が除去されることの検証。
