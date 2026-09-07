#![forbid(unsafe_code)]

use std::convert::Infallible;

use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod plans;
pub mod users;

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

/// Minimal user shape used only to prove `axum-login` composes with the store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackUser {
    pub id: i64,
}

impl AuthUser for StackUser {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &[]
    }
}

/// Backend placeholder for the compile proof. Real authentication is slice 4.
#[derive(Clone, Debug)]
pub struct StackAuthBackend {
    pool: PgPool,
}

impl StackAuthBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl AuthnBackend for StackAuthBackend {
    type User = StackUser;
    type Credentials = ();
    type Error = Infallible;

    async fn authenticate(
        &self,
        _credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
    }

    async fn get_user(&self, _user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
    }
}

pub type StackAuthSession = axum_login::AuthSession<StackAuthBackend>;
