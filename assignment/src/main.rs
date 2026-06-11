mod auth;
mod jwt;
mod models;
mod routes;
mod schema;
mod state;

use anyhow::Context;
use axum::Router;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use state::AppState;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(pool: &diesel::r2d2::Pool<ConnectionManager<PgConnection>>) {
    let mut conn = pool.get().expect("failed to get db connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations failed");
}

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

    run_migrations(&db);

    let redis = redis::Client::open(redis_url).context("failed to create redis client")?;

    let state = AppState { db, redis };

    let app = Router::new()
        .merge(routes::routes())
        .with_state(state);

    println!("Server started on http://0.0.0.0:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("failed to bind address")?;

    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}
