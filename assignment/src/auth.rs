use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

use crate::jwt::validate_token;
use crate::models::AuthUser;
use crate::state::AppState;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AuthUser;

    #[test]
    fn test_require_admin_allows_admin() {
        let user = AuthUser { id: 1, email: "admin@test.com".into(), role: "admin".into() };
        assert!(require_admin(&user).is_ok());
    }

    #[test]
    fn test_require_admin_rejects_staff() {
        let user = AuthUser { id: 2, email: "staff@test.com".into(), role: "staff".into() };
        let err = require_admin(&user).unwrap_err();
        assert!(matches!(err, AuthError::Unauthorized));
    }

    #[test]
    fn test_require_admin_rejects_unknown_role() {
        let user = AuthUser { id: 3, email: "x@test.com".into(), role: "viewer".into() };
        assert!(require_admin(&user).is_err());
    }
}
