use crate::auth::UserId;
use crate::state::AppState;
use axum::extract::State;
use axum::Extension;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/auth/me", get(get_me))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "認証",
    summary = "ログインユーザー情報",
    description = "現在のユーザーのメールアドレスを返す。ヘッダーなしの場合は guest。",
    responses(
        (status = 200, description = "ユーザー情報", body = crate::openapi::UserInfo,
            example = json!({"email": "alice@example.com"})),
    ),
)]
async fn get_me(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> Json<Value> {
    let email = {
        let db = state.db.lock().unwrap();
        db.query_row("SELECT email FROM users WHERE id = ?1", [user_id.0], |row| {
            row.get::<_, String>(0)
        })
        .unwrap_or_else(|_| "guest".to_string())
    };
    Json(json!({ "email": email }))
}
