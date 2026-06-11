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
    #[serde(skip)]
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
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
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse { id: u.id, username: u.username, role: u.role }
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
    pub username: String,
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
    pub username: String,
    pub role: String,
}
