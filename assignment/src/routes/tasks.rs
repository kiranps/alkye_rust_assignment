use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use serde_json::json;

use crate::auth::{authenticate, require_admin, AuthError};
use crate::models::*;
use crate::schema::tasks::dsl::*;
use crate::state::AppState;

async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    require_admin(&user)?;

    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let new_task = NewTask {
        title: body.title,
        description: body.description,
        created_by: user.id,
        assigned_to: body.assigned_to,
        status: "pending".into(),
    };

    let inserted = diesel::insert_into(crate::schema::tasks::table)
        .values(&new_task)
        .returning(Task::as_returning())
        .get_result::<Task>(&mut conn)
        .map_err(|_| AuthError::Internal)?;

    Ok(Json(CreateTaskResponse {
        id: inserted.id,
        title: inserted.title,
        status: inserted.status,
    }))
}

async fn list_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Task>>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let results = if user.role == "admin" {
        tasks.load::<Task>(&mut conn)
    } else {
        tasks.filter(assigned_to.eq(user.id)).load::<Task>(&mut conn)
    };

    let list = results.map_err(|_| AuthError::Internal)?;
    Ok(Json(list))
}

async fn view_my_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Task>>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let list = tasks
        .filter(assigned_to.eq(user.id))
        .load::<Task>(&mut conn)
        .map_err(|_| AuthError::Internal)?;

    Ok(Json(list))
}

async fn assign_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AssignTaskRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    require_admin(&user)?;

    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let updated = diesel::update(tasks.find(body.task_id))
        .set(assigned_to.eq(body.user_id))
        .execute(&mut conn)
        .map_err(|_| AuthError::Internal)?;

    if updated == 0 {
        return Err(AuthError::Unauthorized);
    }

    Ok(Json(json!({ "message": "Task assigned successfully" })))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/assign", post(assign_task))
        .route("/tasks/view-my-tasks", get(view_my_tasks))
}
