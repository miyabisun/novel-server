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
        .get("tailscale-user-login")
        .and_then(|v| v.to_str().ok())
    {
        Some(email) => get_or_create_user(&db, email),
        None => UserId(1), // guest
    };
    request.extensions_mut().insert(user_id);
    next.run(request).await
}

fn get_or_create_user(db: &Arc<Mutex<Connection>>, email: &str) -> UserId {
    let conn = db.lock().expect("lock holders never panic");
    conn.execute("INSERT OR IGNORE INTO users (email) VALUES (?1)", [email])
        .ok();
    let id = conn
        .query_row("SELECT id FROM users WHERE email = ?1", [email], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(1); // fallback to guest
    UserId(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use axum::routing::get;
    use axum::{Extension, Router};
    use tower::ServiceExt;

    async fn whoami(Extension(user_id): Extension<UserId>) -> String {
        user_id.0.to_string()
    }

    async fn user_id_for(db: &Arc<Mutex<Connection>>, header: Option<(&str, &str)>) -> i64 {
        let app = Router::new().route("/whoami", get(whoami)).layer(
            axum::middleware::from_fn_with_state(db.clone(), resolve_user),
        );
        let mut request = Request::builder().uri("/whoami");
        if let Some((name, value)) = header {
            request = request.header(name, value);
        }
        let response = app
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap().parse().unwrap()
    }

    fn memory_db() -> Arc<Mutex<Connection>> {
        Arc::new(Mutex::new(crate::db::open(":memory:")))
    }

    #[tokio::test]
    async fn tailscale_login_header_selects_the_users_own_row() {
        let db = memory_db();
        let login = Some(("Tailscale-User-Login", "alice@example.com"));

        let first = user_id_for(&db, login).await;
        assert_ne!(first, 1, "a signed-in user must not share the guest data");
        assert_eq!(user_id_for(&db, login).await, first);
        let email: String = db
            .lock()
            .unwrap()
            .query_row("SELECT email FROM users WHERE id = ?1", [first], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(email, "alice@example.com");
    }

    #[tokio::test]
    async fn request_without_the_login_header_uses_the_guest() {
        assert_eq!(user_id_for(&memory_db(), None).await, 1);
    }

    #[tokio::test]
    async fn retired_cloudflare_access_header_no_longer_identifies_anyone() {
        // The site is no longer behind Cloudflare Access; any client can send this.
        let header = Some(("Cf-Access-Authenticated-User-Email", "alice@example.com"));
        assert_eq!(user_id_for(&memory_db(), header).await, 1);
    }
}
