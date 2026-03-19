use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
pub struct UserId(pub i64);

pub async fn resolve_user(
    State(db): State<Arc<Mutex<Connection>>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let user_id = match headers
        .get("x-forwarded-email")
        .and_then(|v| v.to_str().ok())
    {
        Some(email) => get_or_create_user(&db, email),
        None => UserId(1), // guest
    };
    request.extensions_mut().insert(user_id);
    next.run(request).await
}

fn get_or_create_user(db: &Arc<Mutex<Connection>>, email: &str) -> UserId {
    let conn = db.lock().unwrap();
    conn.execute("INSERT OR IGNORE INTO users (email) VALUES (?1)", [email])
        .ok();
    let id = conn
        .query_row(
            "SELECT id FROM users WHERE email = ?1",
            [email],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1); // fallback to guest
    UserId(id)
}
