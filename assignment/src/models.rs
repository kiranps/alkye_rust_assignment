use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{tasks, users};

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_by: i32,
    pub assigned_to: Option<i32>,
    pub status: String,
    pub priority: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub created_by: i32,
    pub assigned_to: Option<i32>,
    pub status: String,
    pub priority: String,
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse { id: u.id, email: u.email, role: u.role }
    }
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Verify2faRequest {
    pub code: String,
}

#[derive(Serialize, Debug)]
pub struct Verify2faResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailLogEntry {
    pub code: String,
    pub user_id: i32,
    pub email: String,
    pub sent_at: String,
}

#[derive(Serialize, Debug)]
pub struct EmailLogsResponse {
    pub logs: Vec<EmailLogEntry>,
}

#[derive(Deserialize, Debug)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<i32>,
    pub priority: String,
}

#[derive(Deserialize, Debug)]
pub struct AssignTaskRequest {
    pub task_id: i32,
    pub user_id: i32,
}

#[derive(Serialize, Debug)]
pub struct CreateTaskResponse {
    pub id: i32,
    pub title: String,
    pub status: String,
}

#[derive(Serialize, Debug)]
pub struct SeedResponse {
    pub message: String,
    pub users: Vec<UserResponse>,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub role: String,
}

// new response types for view-my-tasks

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskView {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub total_assigned_tasks: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheInfo {
    pub hit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewMyTasksResponse {
    pub user: UserInfo,
    pub tasks: Vec<TaskView>,
    pub summary: Summary,
    pub cache: CacheInfo,
}
