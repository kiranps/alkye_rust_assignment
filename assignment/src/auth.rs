use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

use crate::jwt::validate_token;
use crate::models::AuthUser;
use crate::state::AppState;

pub enum AuthError {
    Unauthorized,
    Internal,
    TokenCreation,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            AuthError::TokenCreation => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token")
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub async fn authenticate(headers: &HeaderMap, _state: &AppState) -> Result<AuthUser, AuthError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::Unauthorized)?;

    let claims = validate_token(token)?;

    Ok(AuthUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
    })
}

pub fn require_admin(user: &AuthUser) -> Result<(), AuthError> {
    if user.role != "admin" {
        return Err(AuthError::Unauthorized);
    }
    Ok(())
}
