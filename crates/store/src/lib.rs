#![forbid(unsafe_code)]

use sqlx::PgPool;

pub mod assets;
pub mod plans;
pub mod reset_tokens;
mod tokens;
pub mod users;
pub mod verification_tokens;

pub use plans::StoreError;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

/// Runs all migrations, then repeats the original slice-1 connectivity probe.
pub async fn migrate_and_probe(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM stack_probe"#)
        .fetch_one(pool)
        .await
}

pub use users::{AuthBackend, AuthCredentials, AuthSession, User};
