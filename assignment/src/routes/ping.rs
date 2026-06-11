use serde_json::json;

pub async fn ping_handler() -> impl axum::response::IntoResponse {
    axum::Json(json!({ "message": "pong" }))
}
