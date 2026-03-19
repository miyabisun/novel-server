use serde::Serialize;
use utoipa::ToSchema;

/// エラーレスポンス
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// エラーメッセージ
    pub error: String,
}

/// ランキング内の小説情報
#[derive(Serialize, ToSchema)]
pub struct RankingItem {
    /// 小説ID（例: "n1234ab", "16817330666735070954"）
    pub id: String,
    /// 小説タイトル
    pub title: String,
    /// 総ページ数（話数）
    pub page: u64,
    /// 小説種別（1 = 連載, 2 = 短編）。なろう・ノクターンのみ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noveltype: Option<u64>,
}

/// 検索結果の小説情報
#[derive(Serialize, ToSchema)]
pub struct SearchItem {
    /// 小説ID
    pub id: String,
    /// 小説タイトル
    pub title: String,
    /// 総ページ数（話数）
    pub page: u64,
}

/// 小説の詳細情報
#[derive(Serialize, ToSchema)]
pub struct DetailResponse {
    /// 小説タイトル
    pub title: String,
    /// あらすじ
    pub synopsis: String,
    /// 総ページ数（話数）
    pub page: u64,
}

/// エピソード情報
#[derive(Serialize, ToSchema)]
pub struct Episode {
    /// エピソード番号（1始まり）
    pub num: u64,
    /// エピソードタイトル
    pub title: String,
}

/// 目次レスポンス
#[derive(Serialize, ToSchema)]
pub struct TocResponse {
    /// 小説タイトル
    pub title: String,
    /// エピソード一覧
    pub episodes: Vec<Episode>,
}

/// ページ本文レスポンス
#[derive(Serialize, ToSchema)]
pub struct PageResponse {
    /// サニタイズ済みHTML本文。許可タグ: p, br, hr, div, span, h1-h6, ruby, rt, rp, rb, em, strong, b, i, u, s, sub, sup。全属性は除去される
    pub html: String,
}

/// お気に入り情報
#[derive(Serialize, ToSchema)]
pub struct Favorite {
    /// サイト種別（narou / nocturne / kakuyomu）
    #[serde(rename = "type")]
    pub type_str: String,
    /// 小説ID
    pub id: String,
    /// 小説タイトル
    pub title: String,
    /// 小説の更新日時（ISO 8601形式、nullable）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novelupdated_at: Option<String>,
    /// 総ページ数
    pub page: i64,
    /// 既読ページ番号（0 = 未読）
    pub read: i64,
}

/// お気に入り登録リクエスト
#[derive(Serialize, ToSchema)]
pub struct FavoriteRequest {
    /// 小説タイトル
    pub title: String,
    /// 総ページ数
    pub page: i64,
    /// 小説の更新日時（ISO 8601形式、省略可）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub novelupdated_at: Option<String>,
}

/// 既読位置更新リクエスト
#[derive(Serialize, ToSchema)]
pub struct ProgressRequest {
    /// 既読ページ番号
    pub read: i64,
}

/// 成功レスポンス
#[derive(Serialize, ToSchema)]
pub struct OkResponse {
    pub ok: bool,
}

/// ユーザー情報
#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    /// メールアドレス（guest の場合は "guest"）
    pub email: String,
}
