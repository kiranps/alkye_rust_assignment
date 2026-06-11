mod routes;
mod state;

use anyhow::Context;
use axum::routing::get;
use axum::Router;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db = diesel::r2d2::Pool::builder()
        .build(manager)
        .context("failed to create database connection pool")?;

    let redis = redis::Client::open(redis_url).context("failed to create redis client")?;

    let state = AppState { db, redis };

    let app = Router::new()
        .route("/ping", get(routes::ping::ping_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind address")?;

    println!("Server started on http://0.0.0.0:3000");

    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}
