use axum::extract::State;
use axum::Json;

use crate::auth::AuthError;
use crate::models::{EmailLogEntry, EmailLogsResponse};
use crate::state::AppState;

pub async fn email_logs_handler(
    State(state): State<AppState>,
) -> Result<Json<EmailLogsResponse>, AuthError> {
    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let raw: Vec<String> = redis::cmd("LRANGE")
        .arg("email_logs")
        .arg("0")
        .arg("-1")
        .query_async(&mut redis)
        .await
        .map_err(|_| AuthError::Internal)?;

    let logs: Vec<EmailLogEntry> = raw
        .iter()
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect();

    Ok(Json(EmailLogsResponse { logs }))
}
