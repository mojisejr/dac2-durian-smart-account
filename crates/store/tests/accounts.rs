use axum_login::AuthnBackend;
use sqlx::{PgPool, Row};
use store::{
    AuthBackend, AuthCredentials, StoreError, reset_tokens,
    users::{self},
    verification_tokens,
};

const PASSWORD: &str = "correct horse battery staple";
const NEW_PASSWORD: &str = "orchard ledger new password";

#[sqlx::test(migrations = "../../migrations")]
async fn canonical_email_is_unique_even_for_concurrent_registration(
    pool: PgPool,
) -> Result<(), StoreError> {
    let first = users::register(&pool, " Somchai@Example.COM ", PASSWORD);
    let second = users::register(&pool, "somchai@example.com", PASSWORD);
    let (first, second) = tokio::join!(first, second);

    assert!(first.is_ok() ^ second.is_ok());
    let rejected = if first.is_err() { first } else { second };
    assert!(matches!(rejected, Err(StoreError::DuplicateEmail)));

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email_canonical = 'somchai@example.com'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(count, 1);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn verification_is_hashed_single_use_and_resend_invalidates_the_old_link(
    pool: PgPool,
) -> Result<(), StoreError> {
    let user = users::register(&pool, "owner@example.test", PASSWORD).await?;
    let backend = AuthBackend::new(pool.clone());
    let credentials = || AuthCredentials {
        email: "OWNER@example.test".into(),
        password: PASSWORD.into(),
    };
    assert!(backend.authenticate(credentials()).await?.is_none());

    let first = verification_tokens::issue(&pool, user.id).await?;
    let second = verification_tokens::issue(&pool, user.id).await?;
    assert!(matches!(
        verification_tokens::consume(&pool, first.expose()).await,
        Err(StoreError::InvalidToken)
    ));

    let row = sqlx::query("SELECT token_hash FROM email_verification_tokens WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&pool)
        .await?;
    let stored_hash: Vec<u8> = row.try_get("token_hash")?;
    assert_eq!(stored_hash.len(), 32);
    assert_ne!(stored_hash, second.expose().as_bytes());

    assert_eq!(
        verification_tokens::consume(&pool, second.expose()).await?,
        user.id
    );
    assert!(matches!(
        verification_tokens::consume(&pool, second.expose()).await,
        Err(StoreError::InvalidToken)
    ));
    assert!(backend.authenticate(credentials()).await?.is_some());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn expired_altered_and_wrong_purpose_tokens_are_refused(
    pool: PgPool,
) -> Result<(), StoreError> {
    let user = users::register(&pool, "owner@example.test", PASSWORD).await?;
    let verification = verification_tokens::issue(&pool, user.id).await?;
    sqlx::query(
        "UPDATE email_verification_tokens SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
         WHERE user_id = $1",
    )
    .bind(user.id)
    .execute(&pool)
    .await?;
    assert!(matches!(
        verification_tokens::consume(&pool, verification.expose()).await,
        Err(StoreError::InvalidToken)
    ));

    let reset = reset_tokens::issue(&pool, user.id).await?;
    assert!(matches!(
        verification_tokens::consume(&pool, reset.expose()).await,
        Err(StoreError::InvalidToken)
    ));
    let valid_verification = verification_tokens::issue(&pool, user.id).await?;
    assert!(matches!(
        reset_tokens::consume(&pool, valid_verification.expose(), NEW_PASSWORD).await,
        Err(StoreError::InvalidToken)
    ));
    assert!(matches!(
        reset_tokens::consume(&pool, "altered", NEW_PASSWORD).await,
        Err(StoreError::InvalidToken)
    ));

    sqlx::query(
        "UPDATE password_reset_tokens SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
         WHERE user_id = $1",
    )
    .bind(user.id)
    .execute(&pool)
    .await?;
    assert!(matches!(
        reset_tokens::consume(&pool, reset.expose(), NEW_PASSWORD).await,
        Err(StoreError::InvalidToken)
    ));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn password_reset_changes_the_auth_hash_and_invalidates_the_old_password(
    pool: PgPool,
) -> Result<(), StoreError> {
    let user = users::register(&pool, "owner@example.test", PASSWORD).await?;
    let verification = verification_tokens::issue(&pool, user.id).await?;
    verification_tokens::consume(&pool, verification.expose()).await?;
    let before = users::find_by_id(&pool, user.id)
        .await?
        .expect("user exists");
    assert_ne!(before.password_hash, PASSWORD);

    let reset = reset_tokens::issue(&pool, user.id).await?;
    let stored_reset_hash: Vec<u8> =
        sqlx::query_scalar("SELECT token_hash FROM password_reset_tokens WHERE user_id = $1")
            .bind(user.id)
            .fetch_one(&pool)
            .await?;
    assert_ne!(stored_reset_hash, reset.expose().as_bytes());
    reset_tokens::consume(&pool, reset.expose(), NEW_PASSWORD).await?;
    let after = users::find_by_id(&pool, user.id)
        .await?
        .expect("user exists");
    assert_ne!(before.password_hash, after.password_hash);

    let backend = AuthBackend::new(pool);
    assert!(
        backend
            .authenticate(AuthCredentials {
                email: user.email.clone(),
                password: PASSWORD.into(),
            })
            .await?
            .is_none()
    );
    assert!(
        backend
            .authenticate(AuthCredentials {
                email: user.email,
                password: NEW_PASSWORD.into(),
            })
            .await?
            .is_some()
    );
    Ok(())
}
