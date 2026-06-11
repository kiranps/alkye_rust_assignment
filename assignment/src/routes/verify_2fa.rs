use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::auth::AuthError;
use crate::jwt::create_token;
use crate::models::{UserResponse, Verify2faRequest, Verify2faResponse};
use crate::state::AppState;

pub async fn verify_2fa_handler(
    State(state): State<AppState>,
    Json(body): Json<Verify2faRequest>,
) -> Result<Json<Verify2faResponse>, AuthError> {
    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let code_key = format!("2fa_code:{}", body.code);

    let raw: Option<String> = redis::cmd("GET")
        .arg(&code_key)
        .query_async(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    let data: Value = raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .ok_or(AuthError::Unauthorized)?;

    let user_id = data["user_id"].as_i64().ok_or(AuthError::Unauthorized)? as i32;
    let username = data["username"].as_str().ok_or(AuthError::Unauthorized)?;
    let role = data["role"].as_str().ok_or(AuthError::Unauthorized)?;

    let _: () = redis::cmd("DEL")
        .arg(&code_key)
        .query_async(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    let token = create_token(user_id, username, role)?;

    Ok(Json(Verify2faResponse {
        token,
        user: UserResponse {
            id: user_id,
            username: username.to_string(),
            role: role.to_string(),
        },
    }))
}
