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

async fn get_cached_response(redis: &mut impl AsyncCommands, key: &str) -> Result<Option<ViewMyTasksResponse>, AuthError> {
    let data: Option<String> = redis.get(key).await.map_err(|_| AuthError::Internal)?;
    match data {
        Some(raw) => serde_json::from_str(&raw).map(Some).map_err(|_| AuthError::Internal),
        None => Ok(None),
    }
}

async fn set_cached_response(redis: &mut impl AsyncCommands, key: &str, resp: &ViewMyTasksResponse) -> Result<(), AuthError> {
    let raw = serde_json::to_string(resp).map_err(|_| AuthError::Internal)?;
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

fn make_task_view(task: &Task, email: &str) -> TaskView {
    TaskView {
        id: task.id,
        title: task.title.clone(),
        status: task.status.clone(),
        priority: task.priority.clone(),
        assigned_to: email.to_string(),
    }
}

fn make_response(user: &AuthUser, task_list: &[Task], cache_hit: bool) -> ViewMyTasksResponse {
    let task_views: Vec<TaskView> = task_list
        .iter()
        .map(|t| make_task_view(t, &user.email))
        .collect();

    ViewMyTasksResponse {
        user: UserInfo {
            email: user.email.clone(),
            role: user.role.clone(),
        },
        tasks: task_views,
        summary: Summary {
            total_assigned_tasks: task_list.len(),
        },
        cache: CacheInfo { hit: cache_hit },
    }
}

fn build_response(user: &AuthUser, task_list: &[Task]) -> ViewMyTasksResponse {
    make_response(user, task_list, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn task_fixture(task_id: i32, assigned: Option<i32>) -> Task {
        Task {
            id: task_id,
            title: format!("Task {}", task_id),
            description: None,
            created_by: 1,
            assigned_to: assigned,
            status: "todo".into(),
            priority: "medium".into(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    fn user_fixture(user_id: i32, role: &str) -> AuthUser {
        AuthUser {
            id: user_id,
            email: format!("{}@test.com", role),
            role: role.into(),
        }
    }

    #[test]
    fn test_cache_key_user() {
        assert_eq!(cache_key_user(42), "tasks:user:42");
        assert_eq!(cache_key_user(0), "tasks:user:0");
        assert_eq!(cache_key_user(999), "tasks:user:999");
    }

    #[test]
    fn test_make_task_view() {
        let task = task_fixture(1, Some(5));
        let view = make_task_view(&task, "user@example.com");

        assert_eq!(view.id, 1);
        assert_eq!(view.title, "Task 1");
        assert_eq!(view.status, "todo");
        assert_eq!(view.priority, "medium");
        assert_eq!(view.assigned_to, "user@example.com");
    }

    #[test]
    fn test_make_response_with_tasks() {
        let user = user_fixture(1, "staff");
        let list = vec![task_fixture(10, Some(1)), task_fixture(20, Some(1))];
        let resp = make_response(&user, &list, false);

        assert_eq!(resp.user.email, "staff@test.com");
        assert_eq!(resp.user.role, "staff");
        assert_eq!(resp.tasks.len(), 2);
        assert_eq!(resp.tasks[0].id, 10);
        assert_eq!(resp.tasks[1].id, 20);
        assert_eq!(resp.summary.total_assigned_tasks, 2);
        assert!(!resp.cache.hit);
    }

    #[test]
    fn test_make_response_empty_tasks() {
        let user = user_fixture(2, "admin");
        let resp = make_response(&user, &[], true);

        assert_eq!(resp.user.email, "admin@test.com");
        assert!(resp.tasks.is_empty());
        assert_eq!(resp.summary.total_assigned_tasks, 0);
        assert!(resp.cache.hit);
    }

    #[test]
    fn test_build_response_sets_cache_miss() {
        let user = user_fixture(1, "staff");
        let resp = build_response(&user, &[task_fixture(1, Some(1))]);
        assert!(!resp.cache.hit);
    }

    #[test]
    fn test_make_task_view_preserves_all_fields() {
        let task = Task {
            id: 99,
            title: "Urgent bug".into(),
            description: Some("fix it".into()),
            created_by: 2,
            assigned_to: Some(3),
            status: "in_progress".into(),
            priority: "high".into(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };
        let view = make_task_view(&task, "dev@example.com");

        assert_eq!(view.id, 99);
        assert_eq!(view.title, "Urgent bug");
        assert_eq!(view.status, "in_progress");
        assert_eq!(view.priority, "high");
        assert_eq!(view.assigned_to, "dev@example.com");
    }
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
        status: "todo".into(),
        priority: body.priority,
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
) -> Result<Json<ViewMyTasksResponse>, AuthError> {
    let user = authenticate(&headers, &state).await?;
    let key = cache_key_user(user.id);

    let mut redis = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AuthError::Internal)?;

    if let Some(cached) = get_cached_response(&mut redis, &key).await? {
        return Ok(Json(cached));
    }

    let mut conn = state.db.get().map_err(|_| AuthError::Internal)?;

    let list = tasks
        .filter(assigned_to.eq(user.id))
        .load::<Task>(&mut conn)
        .map_err(|_| AuthError::Internal)?;

    let resp = build_response(&user, &list);
    set_cached_response(&mut redis, &key, &resp).await?;

    Ok(Json(resp))
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
