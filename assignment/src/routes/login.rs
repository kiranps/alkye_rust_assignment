use axum::extract::State;
use axum::Json;
use diesel::prelude::*;

use crate::auth::{generate_token, AuthError};
use crate::models::{LoginRequest, LoginResponse, User, UserResponse};
use crate::schema::users::dsl::*;
use crate::state::AppState;

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

    let token = generate_token();

    let mut redis_conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let key = format!("session:{}", token);
    let val = user.id.to_string();
    redis::cmd("SETEX")
        .arg(&key)
        .arg(&val)
        .arg("86400")
        .query_async::<()>(&mut redis_conn)
        .await
        .map_err(|_| AuthError::Internal)?;

    Ok(Json(LoginResponse {
        token,
        user: UserResponse::from(user),
    }))
}
