use sqlx::{PgPool, Row};

use crate::plans::StoreError;

pub type UserId = i64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub id: UserId,
    pub email: String,
}

/// Create the ownership identity used by persistence tests and later auth work.
/// Password and session semantics intentionally remain in slice 4.
pub async fn create(pool: &PgPool, email: &str) -> Result<User, StoreError> {
    let row = sqlx::query("INSERT INTO users (email) VALUES ($1) RETURNING id, email")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(User {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
    })
}
