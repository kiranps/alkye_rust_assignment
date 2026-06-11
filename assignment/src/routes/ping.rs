use axum::routing::get;
use axum::Router;
use serde_json::json;

use crate::state::AppState;

async fn ping_handler() -> impl axum::response::IntoResponse {
    axum::Json(json!({ "message": "pong" }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/ping", get(ping_handler))
}
