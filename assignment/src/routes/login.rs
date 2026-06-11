use axum::extract::State;
use axum::Json;
use diesel::prelude::*;

use crate::auth::AuthError;
use crate::jwt::create_token;
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

    let token = create_token(user.id, &user.username, &user.role)?;

    Ok(Json(LoginResponse {
        token,
        user: UserResponse::from(user),
    }))
}
