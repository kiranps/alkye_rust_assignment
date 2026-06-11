use axum::extract::State;
use axum::Json;
use diesel::prelude::*;
use serde_json::json;

use crate::auth::AuthError;
use crate::models::{EmailLogEntry, LoginRequest, LoginResponse, User};
use crate::schema::users::dsl::*;
use crate::state::AppState;

fn generate_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.r#gen::<u32>() % 1_000_000)
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let user = users
        .filter(username.eq(&body.username))
        .first::<User>(&mut conn)
        .map_err(|_| AuthError::Unauthorized)?;

    if user.password != body.password {
        return Err(AuthError::Unauthorized);
    }

    let code = generate_code();

    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let code_key = format!("2fa_code:{}", code);
    let user_data = json!({
        "user_id": user.id,
        "username": user.username,
        "role": user.role,
    });

    redis::cmd("SETEX")
        .arg(&code_key)
        .arg("300")
        .arg(user_data.to_string())
        .query_async::<()>(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let log_entry = EmailLogEntry {
        code: code.clone(),
        user_id: user.id,
        username: user.username.clone(),
        sent_at: now.to_string(),
    };

    let log_json = serde_json::to_string(&log_entry).map_err(|_| AuthError::Internal)?;

    redis::cmd("LPUSH")
        .arg("email_logs")
        .arg(&log_json)
        .query_async::<()>(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    redis::cmd("LTRIM")
        .arg("email_logs")
        .arg("0")
        .arg("99")
        .query_async::<()>(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    Ok(Json(LoginResponse {
        message: "Verification code sent to your email".into(),
    }))
}
