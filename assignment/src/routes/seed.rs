use axum::extract::State;
use axum::Json;
use diesel::prelude::*;

use crate::auth::AuthError;
use crate::models::{NewUser, SeedResponse, User, UserResponse};
use crate::schema::users::dsl::*;
use crate::state::AppState;

pub async fn seed_users_handler(
    State(state): State<AppState>,
) -> Result<Json<SeedResponse>, AuthError> {
    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let users_to_create = vec![
        NewUser { username: "admin".into(), password: "admin123".into(), role: "admin".into() },
        NewUser { username: "james_bond".into(), password: "bond123".into(), role: "user".into() },
    ];

    for u in &users_to_create {
        let _ = diesel::insert_into(crate::schema::users::table)
            .values(u)
            .execute(&mut conn);
    }

    let all = users.load::<User>(&mut conn).map_err(|_| AuthError::Internal)?;
    let users_resp: Vec<UserResponse> = all.into_iter().map(Into::into).collect();

    Ok(Json(SeedResponse {
        message: "Seed users created".into(),
        users: users_resp,
    }))
}
