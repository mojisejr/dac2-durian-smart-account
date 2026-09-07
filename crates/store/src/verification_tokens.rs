use sqlx::PgPool;

use crate::{StoreError, tokens, users::UserId};

pub use crate::tokens::IssuedToken;

pub async fn issue(pool: &PgPool, user_id: UserId) -> Result<IssuedToken, StoreError> {
    let token = tokens::issue();
    let hash = tokens::hash(token.expose());
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(transaction.as_mut())
        .await?;
    sqlx::query(
        "INSERT INTO email_verification_tokens (user_id, token_hash, expires_at) \
         VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '24 hours')",
    )
    .bind(user_id)
    .bind(hash)
    .execute(transaction.as_mut())
    .await?;
    transaction.commit().await?;
    Ok(token)
}

pub async fn consume(pool: &PgPool, raw: &str) -> Result<UserId, StoreError> {
    let hash = tokens::hash(raw);
    let mut transaction = pool.begin().await?;
    let user_id = sqlx::query_scalar::<_, UserId>(
        "DELETE FROM email_verification_tokens \
         WHERE token_hash = $1 AND expires_at > CURRENT_TIMESTAMP \
         RETURNING user_id",
    )
    .bind(hash)
    .fetch_optional(transaction.as_mut())
    .await?
    .ok_or(StoreError::InvalidToken)?;
    sqlx::query("UPDATE users SET email_verified_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(user_id)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    Ok(user_id)
}
