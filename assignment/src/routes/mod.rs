pub mod login;
pub mod ping;
pub mod seed;
pub mod tasks;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ping", get(ping::ping_handler))
        .route("/auth/login", post(login::login_handler))
        .route("/seed/users", post(seed::seed_users_handler))
        .route("/tasks", get(tasks::list_tasks).post(tasks::create_task))
        .route("/tasks/assign", post(tasks::assign_task))
        .route("/tasks/view-my-tasks", get(tasks::view_my_tasks))
}
