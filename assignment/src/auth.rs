use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use diesel::prelude::*;
use serde_json::json;

use crate::models::{AuthUser, User};
use crate::schema::users::dsl::*;
use crate::state::AppState;

pub enum AuthError {
    Unauthorized,
    Internal,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<AuthUser, AuthError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::Unauthorized)?;

    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let uid: Option<String> = redis::cmd("GET")
        .arg(&[format!("session:{}", token)])
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| AuthError::Internal)?;

    let uid: i32 = uid
        .and_then(|s| s.parse().ok())
        .ok_or(AuthError::Unauthorized)?;

    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let user = users
        .find(uid)
        .first::<User>(&mut conn)
        .map_err(|_| AuthError::Unauthorized)?;

    Ok(AuthUser { id: user.id, username: user.username, role: user.role })
}

pub fn require_admin(user: &AuthUser) -> Result<(), AuthError> {
    if user.role != "admin" {
        return Err(AuthError::Unauthorized);
    }
    Ok(())
}

pub fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32).map(|_| format!("{:02x}", rng.r#gen::<u8>())).collect()
}
