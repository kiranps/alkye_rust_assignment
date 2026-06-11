use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct PingResponse {
    pub message: String,
}

pub async fn ping_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(PingResponse {
        message: "pong".to_string(),
    }))
}
