pub mod auth;
pub mod jwt;
pub mod models;
pub mod routes;
pub mod schema;
pub mod state;

use axum::Router;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(pool: &diesel::r2d2::Pool<ConnectionManager<PgConnection>>) {
    let mut conn = pool
        .get()
        .expect("failed to get db connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("migrations failed");
}

pub fn build_app(
    db: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
    redis: redis::Client,
) -> Router {
    let state = state::AppState { db, redis };
    Router::new()
        .merge(routes::routes())
        .with_state(state)
}
