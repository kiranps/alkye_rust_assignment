use axum::routing::post;
use axum::Router;

use crate::auth::{login_handler, seed_users_handler};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/seed/users", post(seed_users_handler))
}
