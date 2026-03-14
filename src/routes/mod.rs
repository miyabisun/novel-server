mod detail;
mod favorites;
mod pages;
mod ranking;
mod search;
mod toc;

use crate::error::AppError;
use crate::openapi;
use crate::spa;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use std::future::Future;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Novel Server API",
        version = "0.1.0",
        description = "小説家になろう・ノクターンノベルズ・カクヨムの小説を横断的に検索・閲覧するためのAPI。\n\n## キャッシュ戦略\n| 対象 | TTL | 説明 |\n|------|-----|------|\n| ランキング | 3時間 | 各サイトのランキングは頻繁には更新されない |\n| 検索結果 | 1時間 | 新作投稿を早めに反映するため短めのTTL |\n| 小説詳細 | 24時間 | タイトル・あらすじは基本的に変わらない |\n| ページ本文 | 24時間 | 小説の本文は基本的に変わらない |\n| 目次 | なし | リアルタイム性を重視（最新の話数を即時反映） |\n\nキャッシュの強制更新は各エンドポイントのPATCHメソッドで行えます。\n\n## リトライ\n小説詳細・目次・ページ本文の取得は最大3回リトライされます（500ms × 試行回数のバックオフ）。\n\n## HTMLサニタイズ\nページ本文のHTMLは許可リスト方式でサニタイズされます。許可タグ: p, br, hr, div, span, h1-h6, ruby, rt, rp, rb, em, strong, b, i, u, s, sub, sup。全属性は除去されます。",
    ),
    paths(
        ranking::get_ranking,
        ranking::patch_ranking,
        search::get_search,
        detail::get_detail,
        toc::get_toc,
        pages::get_page,
        pages::patch_page,
        favorites::get_favorites,
        favorites::put_favorite,
        favorites::delete_favorite,
        favorites::patch_progress,
    ),
    components(schemas(
        openapi::ErrorResponse,
        openapi::RankingItem,
        openapi::SearchItem,
        openapi::DetailResponse,
        openapi::Episode,
        openapi::TocResponse,
        openapi::PageResponse,
        openapi::Favorite,
        openapi::FavoriteRequest,
        openapi::ProgressRequest,
        openapi::OkResponse,
    )),
    tags(
        (name = "ランキング", description = "ランキング取得・再取得"),
        (name = "検索", description = "小説のキーワード検索"),
        (name = "小説情報", description = "小説の詳細情報・目次の取得"),
        (name = "小説本文", description = "小説の本文HTML取得"),
        (name = "お気に入り", description = "お気に入りのCRUD操作・既読管理"),
    ),
)]
struct ApiDoc;

/// リトライはルートハンドラ層のみが担う。モジュール層は単純な fetch に徹し、
/// バックグラウンド同期はループ継続で暗黙的に再試行される。
async fn with_retry<F, Fut, T>(label: &str, f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    for i in 0..3u32 {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                tracing::error!("{} attempt {} failed: {}", label, i + 1, e);
                if i < 2 {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * (i as u64 + 1)))
                        .await;
                }
            }
        }
    }
    Err(AppError::Upstream(format!(
        "Failed after 3 retries: {}",
        label
    )))
}

pub fn build_router(state: AppState) -> Router {
    let base_path = state.config.base_path.clone();

    let api = Router::new()
        .merge(ranking::routes())
        .merge(pages::routes())
        .merge(detail::routes())
        .merge(favorites::routes())
        .merge(search::routes())
        .merge(toc::routes());

    let sub = Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest_service("/assets", ServeDir::new("client/build/assets"))
        .nest_service(
            "/favicon.svg",
            tower_http::services::ServeFile::new("client/build/favicon.svg"),
        )
        .fallback(get(move || {
            let bp = base_path.clone();
            async move { spa_fallback(&bp) }
        }))
        .with_state(state.clone());

    let app_base = state.config.base_path.clone();
    if app_base.is_empty() {
        sub
    } else {
        Router::new().nest(&app_base, sub)
    }
}

fn spa_fallback(base_path: &str) -> impl IntoResponse {
    match spa::get_index_html(base_path) {
        Some(html) => Html(html).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "Frontend not built. Run: cd client && npm install && npx vite build"})),
        )
            .into_response(),
    }
}
