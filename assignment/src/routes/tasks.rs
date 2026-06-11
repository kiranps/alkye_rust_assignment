use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use diesel::prelude::*;
use redis::AsyncCommands;
use serde_json::json;

use crate::auth::{authenticate, require_admin, AuthError};
use crate::models::*;
use crate::schema::tasks::dsl::*;
use crate::state::AppState;

const CACHE_TTL: u64 = 30;

fn cache_key_user(uid: i32) -> String {
    format!("tasks:user:{}", uid)
}

async fn get_cached_tasks(redis: &mut impl AsyncCommands, key: &str) -> Result<Option<Vec<Task>>, AuthError> {
    let data: Option<String> = redis.get(key).await.map_err(|_| AuthError::Internal)?;
    match data {
        Some(raw) => serde_json::from_str(&raw).map(Some).map_err(|_| AuthError::Internal),
        None => Ok(None),
    }
}

async fn set_cached_tasks(redis: &mut impl AsyncCommands, key: &str, items: &[Task]) -> Result<(), AuthError> {
    let raw = serde_json::to_string(items).map_err(|_| AuthError::Internal)?;
    let _: () = redis.set_ex(key, raw, CACHE_TTL).await.map_err(|_| AuthError::Internal)?;
    Ok(())
}

async fn invalidate_task_caches(state: &AppState) -> Result<(), AuthError> {
    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    let keys: Vec<String> = redis
        .keys("tasks:*")
        .await
        .map_err(|_| AuthError::Internal)?;

    if !keys.is_empty() {
        let _: () = redis.del(keys).await.map_err(|_| AuthError::Internal)?;
    }

    Ok(())
}

pub async fn create_task(
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

    invalidate_task_caches(&state).await?;

    Ok(Json(CreateTaskResponse {
        id: inserted.id,
        title: inserted.title,
        status: inserted.status,
    }))
}

pub async fn view_my_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Task>>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    let key = cache_key_user(user.id);

    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    if let Some(cached) = get_cached_tasks(&mut redis, &key).await? {
        return Ok(Json(cached));
    }

    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let list = tasks
        .filter(assigned_to.eq(user.id))
        .load::<Task>(&mut conn)
        .map_err(|_| AuthError::Internal)?;

    set_cached_tasks(&mut redis, &key, &list).await?;

    Ok(Json(list))
}

pub async fn assign_task(
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

    invalidate_task_caches(&state).await?;

    Ok(Json(json!({ "message": "Task assigned successfully" })))
}
