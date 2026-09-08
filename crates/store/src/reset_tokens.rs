use sqlx::PgPool;

use crate::{StoreError, tokens, users, users::UserId};

pub use crate::tokens::IssuedToken;

pub async fn issue(pool: &PgPool, user_id: UserId) -> Result<IssuedToken, StoreError> {
    let token = tokens::issue();
    let hash = tokens::hash(token.expose());
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(transaction.as_mut())
        .await?;
    sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) \
         VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '1 hour')",
    )
    .bind(user_id)
    .bind(hash)
    .execute(transaction.as_mut())
    .await?;
    transaction.commit().await?;
    Ok(token)
}

pub async fn consume(pool: &PgPool, raw: &str, new_password: &str) -> Result<UserId, StoreError> {
    let password_hash = users::hash_password(new_password)?;
    let token_hash = tokens::hash(raw);
    let mut transaction = pool.begin().await?;
    let user_id = sqlx::query_scalar::<_, UserId>(
        "DELETE FROM password_reset_tokens \
         WHERE token_hash = $1 AND expires_at > CURRENT_TIMESTAMP \
         RETURNING user_id",
    )
    .bind(token_hash)
    .fetch_optional(transaction.as_mut())
    .await?
    .ok_or(StoreError::InvalidToken)?;
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
        .bind(user_id)
        .bind(password_hash)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    Ok(user_id)
}
