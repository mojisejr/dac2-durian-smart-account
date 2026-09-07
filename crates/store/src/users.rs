use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier},
};
use axum_login::{AuthUser, AuthnBackend, UserId as BackendUserId};
use email_address::EmailAddress;
use sqlx::{PgPool, Row};

use crate::StoreError;

pub type UserId = i64;

const MIN_PASSWORD_CHARS: usize = 15;
const MAX_PASSWORD_CHARS: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailIdentity {
    pub original: String,
    pub canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub email_canonical: String,
    pub password_hash: String,
    pub email_verified: bool,
}

impl AuthUser for User {
    type Id = UserId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Clone, Debug)]
pub struct AuthCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct AuthBackend {
    pool: PgPool,
}

impl AuthBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = AuthCredentials;
    type Error = StoreError;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Ok(identity) = parse_email(&credentials.email) else {
            verify_dummy_password(&credentials.password);
            return Ok(None);
        };

        let user = find_by_canonical(&self.pool, &identity.canonical).await?;
        let Some(user) = user else {
            verify_dummy_password(&credentials.password);
            return Ok(None);
        };

        let password_matches = verify_password(&user.password_hash, &credentials.password);
        if password_matches && user.email_verified {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_user(
        &self,
        user_id: &BackendUserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(find_by_id(&self.pool, *user_id)
            .await?
            .filter(|user| user.email_verified))
    }
}

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub fn parse_email(input: &str) -> Result<EmailIdentity, StoreError> {
    let original = input.trim();
    if original.is_empty() || original.parse::<EmailAddress>().is_err() {
        return Err(StoreError::InvalidEmail);
    }

    Ok(EmailIdentity {
        original: original.to_owned(),
        canonical: original.to_ascii_lowercase(),
    })
}

pub fn validate_password(password: &str) -> Result<(), StoreError> {
    let count = password.chars().count();
    if !(MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&count) {
        return Err(StoreError::InvalidPassword);
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, StoreError> {
    validate_password(password)?;
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|error| StoreError::Crypto(error.to_string()))
}

pub fn verify_password(password_hash: &str, password: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), password_hash)
        .is_ok()
}

/// Creates a non-authenticating identity for persistence tests and fixtures.
/// Production registration must use [`register`].
pub async fn create(pool: &PgPool, email: &str) -> Result<User, StoreError> {
    let identity = parse_email(email)?;
    insert_user(pool, &identity, "!unusable!").await
}

pub async fn register(pool: &PgPool, email: &str, password: &str) -> Result<User, StoreError> {
    let identity = parse_email(email)?;
    let password_hash = hash_password(password)?;
    insert_user(pool, &identity, &password_hash).await
}

async fn insert_user(
    pool: &PgPool,
    identity: &EmailIdentity,
    password_hash: &str,
) -> Result<User, StoreError> {
    let result = sqlx::query(
        "INSERT INTO users (email, email_canonical, password_hash) \
         VALUES ($1, $2, $3) \
         RETURNING id, email, email_canonical, password_hash, \
                   email_verified_at IS NOT NULL AS email_verified",
    )
    .bind(&identity.original)
    .bind(&identity.canonical)
    .bind(password_hash)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => row_to_user(&row),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            Err(StoreError::DuplicateEmail)
        }
        Err(error) => Err(StoreError::Database(error)),
    }
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, StoreError> {
    let identity = parse_email(email)?;
    find_by_canonical(pool, &identity.canonical).await
}

pub async fn find_by_canonical(pool: &PgPool, canonical: &str) -> Result<Option<User>, StoreError> {
    let row = sqlx::query(
        "SELECT id, email, email_canonical, password_hash, \
                email_verified_at IS NOT NULL AS email_verified \
         FROM users WHERE email_canonical = $1",
    )
    .bind(canonical)
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(row_to_user).transpose()
}

pub async fn find_by_id(pool: &PgPool, id: UserId) -> Result<Option<User>, StoreError> {
    let row = sqlx::query(
        "SELECT id, email, email_canonical, password_hash, \
                email_verified_at IS NOT NULL AS email_verified \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    row.as_ref().map(row_to_user).transpose()
}

fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<User, StoreError> {
    Ok(User {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        email_canonical: row.try_get("email_canonical")?,
        password_hash: row.try_get("password_hash")?,
        email_verified: row.try_get("email_verified")?,
    })
}

fn verify_dummy_password(password: &str) {
    const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$AC1yxVygo40ISXmpmD3AmGvX5w2l8b5zGxs44+MFU1s";
    let _ = Argon2::default().verify_password(password.as_bytes(), DUMMY_HASH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_is_trimmed_and_compared_without_ascii_case() {
        let email = parse_email("  Somchai+สวน@Example.COM  ").expect("valid email");
        assert_eq!(email.original, "Somchai+สวน@Example.COM");
        assert_eq!(email.canonical, "somchai+สวน@example.com");
    }

    #[test]
    fn provider_specific_aliases_remain_distinct() {
        let plain = parse_email("somchai@example.com").expect("valid email");
        let tagged = parse_email("somchai+สวน@example.com").expect("valid email");
        let dotted = parse_email("som.chai@example.com").expect("valid email");
        assert_ne!(plain.canonical, tagged.canonical);
        assert_ne!(plain.canonical, dotted.canonical);
    }

    #[test]
    fn malformed_email_is_rejected() {
        for value in ["", "somchai", "@example.com", "somchai@"] {
            assert!(matches!(parse_email(value), Err(StoreError::InvalidEmail)));
        }
    }

    #[test]
    fn password_policy_counts_unicode_characters_and_has_no_composition_rule() {
        assert!(validate_password("ยาวสิบห้าตัวพอดี").is_ok());
        assert!(validate_password("correct horse battery staple").is_ok());
        assert!(validate_password("aaaaaaaaaaaaaa").is_err());
        assert!(validate_password(&"a".repeat(64)).is_ok());
        assert!(validate_password(&"a".repeat(MAX_PASSWORD_CHARS + 1)).is_err());
    }

    #[test]
    fn argon2_hashes_are_salted_and_verify_only_the_original_password() {
        let password = "correct horse battery staple";
        let first = hash_password(password).expect("password hashes");
        let second = hash_password(password).expect("password hashes again");
        assert_ne!(first, second);
        assert!(first.starts_with("$argon2id$"));
        assert!(verify_password(&first, password));
        assert!(!verify_password(
            &first,
            "wrong password that is long enough"
        ));
    }
}
