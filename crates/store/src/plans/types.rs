use std::fmt;

use calc::Plan;

use crate::users::UserId;

pub type PlanId = i64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanSummary {
    pub id: PlanId,
    pub name: String,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredPlan {
    pub id: PlanId,
    pub owner_id: UserId,
    pub closed: bool,
    pub plan: Plan,
}

#[derive(Debug)]
pub enum StoreError {
    Database(sqlx::Error),
    NotFound,
    Closed,
    InvalidEmail,
    InvalidPassword,
    DuplicateEmail,
    InvalidToken,
    Crypto(String),
    InvalidValue { field: &'static str, value: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::NotFound => formatter.write_str("plan not found"),
            Self::Closed => formatter.write_str("closed plan is read-only"),
            Self::InvalidEmail => formatter.write_str("invalid email address"),
            Self::InvalidPassword => formatter.write_str("password does not meet policy"),
            Self::DuplicateEmail => formatter.write_str("email address is already registered"),
            Self::InvalidToken => formatter.write_str("token is invalid or expired"),
            Self::Crypto(error) => write!(formatter, "credential operation failed: {error}"),
            Self::InvalidValue { field, value } => {
                write!(formatter, "invalid stored value for {field}: {value}")
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::NotFound
            | Self::Closed
            | Self::InvalidEmail
            | Self::InvalidPassword
            | Self::DuplicateEmail
            | Self::InvalidToken
            | Self::Crypto(_)
            | Self::InvalidValue { .. } => None,
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}
