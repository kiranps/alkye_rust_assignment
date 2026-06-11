use std::net::TcpListener;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

/// Spawn the application on a random port and return its base URL.
async fn spawn_app() -> String {
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("TEST_DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/assignment".to_string());

    let redis_url = std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("TEST_REDIS_URL"))
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create database pool");

    assignment::run_migrations(&db);

    let redis = redis::Client::open(redis_url).expect("failed to create redis client");

    let app = assignment::build_app(db, redis);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    tokio::task::spawn(async move {
        let tokio_listener = tokio::net::TcpListener::from_std(listener).unwrap();
        axum::serve(tokio_listener, app).await.unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}

/// Fetch all email logs and return the latest 2FA code.
async fn fetch_latest_code(base_url: &str) -> String {
    let resp = client()
        .get(format!("{}/dev/email-logs/latest", base_url))
        .send()
        .await
        .expect("failed to fetch email logs");

    let body: serde_json::Value = resp.json().await.unwrap();
    let logs = body["logs"].as_array().unwrap();
    let latest = logs.last().unwrap();
    latest["code"].as_str().unwrap().to_string()
}

/// Full auth flow: login + verify 2FA -> JWT token.
async fn login_as(base_url: &str, username: &str, password: &str) -> String {
    let resp = client()
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await
        .expect("login request failed");
    assert_eq!(resp.status(), 200, "login should succeed for valid credentials");

    let code = fetch_latest_code(base_url).await;

    let resp = client()
        .post(format!("{}/auth/verify-2fa", base_url))
        .json(&serde_json::json!({ "code": code }))
        .send()
        .await
        .expect("verify 2fa request failed");
    assert_eq!(resp.status(), 200, "2FA verification should succeed");

    let body: serde_json::Value = resp.json().await.unwrap();
    body["token"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Integration tests — require Postgres + Redis running (docker-compose up).
// Run with: cargo test -- --ignored --test-threads=1
// ---------------------------------------------------------------------------

#[ignore]
#[tokio::test]
async fn health_check() {
    let base_url = spawn_app().await;
    let resp = client().get(format!("{}/ping", base_url)).send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[ignore]
#[tokio::test]
async fn seed_users_creates_admin_and_staff() {
    let base_url = spawn_app().await;

    let resp = client()
        .post(format!("{}/seed/users", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["message"], "Seed users created");

    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);

    let emails: Vec<&str> = users.iter().map(|u| u["email"].as_str().unwrap()).collect();
    assert!(emails.contains(&"admin@example.com"));
    assert!(emails.contains(&"jamesbond@example.com"));

    let roles: Vec<&str> = users.iter().map(|u| u["role"].as_str().unwrap()).collect();
    assert!(roles.contains(&"admin"));
    assert!(roles.contains(&"staff"));
}

#[ignore]
#[tokio::test]
async fn full_auth_flow() {
    let base_url = spawn_app().await;

    // seed users
    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();

    // login as admin
    let token = login_as(&base_url, "admin", "admin123").await;
    assert!(!token.is_empty(), "should receive a JWT token");
}

#[ignore]
#[tokio::test]
async fn admin_can_create_and_view_tasks() {
    let base_url = spawn_app().await;

    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();
    let token = login_as(&base_url, "admin", "admin123").await;

    // create a task
    let resp = client()
        .post(format!("{}/tasks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": "Test task",
            "description": "A task for testing",
            "assigned_to": null,
            "priority": "high"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Test task");
    assert_eq!(body["status"], "todo");
    let task_id = body["id"].as_i64().unwrap();

    // view my tasks (admin has none assigned yet)
    let resp = client()
        .get(format!("{}/tasks/view-my-tasks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["user"]["email"], "admin@example.com");
    assert_eq!(body["user"]["role"], "admin");
    assert_eq!(body["summary"]["total_assigned_tasks"], 0);
    assert_eq!(body["cache"]["hit"], false);

    let _ = task_id;
}

#[ignore]
#[tokio::test]
async fn admin_can_assign_task_to_staff() {
    let base_url = spawn_app().await;

    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();
    let token = login_as(&base_url, "admin", "admin123").await;

    // create a task
    let resp = client()
        .post(format!("{}/tasks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": "Assignable task",
            "priority": "medium"
        }))
        .send()
        .await
        .unwrap();
    let task_id = resp.json::<serde_json::Value>().await.unwrap()["id"].as_i64().unwrap();

    // find staff user id via seed response (re-seed to get fresh list)
    let seed_resp = client()
        .post(format!("{}/seed/users", base_url))
        .send()
        .await
        .unwrap();
    let users = seed_resp.json::<serde_json::Value>().await.unwrap()["users"].as_array().unwrap().clone();
    let staff_user = users.iter().find(|u| u["email"] == "jamesbond@example.com").unwrap();
    let staff_id = staff_user["id"].as_i64().unwrap();

    // assign task to staff
    let resp = client()
        .post(format!("{}/tasks/assign", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "task_id": task_id,
            "user_id": staff_id
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // staff logs in and sees the task
    let staff_token = login_as(&base_url, "james_bond", "bond123").await;

    let resp = client()
        .get(format!("{}/tasks/view-my-tasks", base_url))
        .header("Authorization", format!("Bearer {}", staff_token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["user"]["email"], "jamesbond@example.com");
    assert_eq!(body["user"]["role"], "staff");
    assert_eq!(body["summary"]["total_assigned_tasks"], 1);
    assert_eq!(body["tasks"][0]["title"], "Assignable task");
    assert_eq!(body["tasks"][0]["priority"], "medium");
    assert_eq!(body["tasks"][0]["assigned_to"], "jamesbond@example.com");
}

#[ignore]
#[tokio::test]
async fn staff_cannot_create_tasks() {
    let base_url = spawn_app().await;

    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();
    let token = login_as(&base_url, "james_bond", "bond123").await;

    let resp = client()
        .post(format!("{}/tasks", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": "Staff task",
            "priority": "low"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401, "staff should be unauthorized to create tasks");
}

#[ignore]
#[tokio::test]
async fn login_fails_with_wrong_password() {
    let base_url = spawn_app().await;
    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();

    let resp = client()
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({ "username": "admin", "password": "wrongpass" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[ignore]
#[tokio::test]
async fn verify_2fa_fails_with_wrong_code() {
    let base_url = spawn_app().await;
    client().post(format!("{}/seed/users", base_url)).send().await.unwrap();

    // login to trigger code generation
    client()
        .post(format!("{}/auth/login", base_url))
        .json(&serde_json::json!({ "username": "admin", "password": "admin123" }))
        .send()
        .await
        .unwrap();

    // attempt with a bogus code
    let resp = client()
        .post(format!("{}/auth/verify-2fa", base_url))
        .json(&serde_json::json!({ "code": "000000" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[ignore]
#[tokio::test]
async fn endpoint_without_token_is_rejected() {
    let base_url = spawn_app().await;

    let resp = client()
        .get(format!("{}/tasks/view-my-tasks", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}
