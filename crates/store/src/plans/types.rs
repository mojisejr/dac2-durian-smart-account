use std::fmt;

use calc::Plan;

use crate::users::UserId;

pub type PlanId = i64;

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
    InvalidValue { field: &'static str, value: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "database error: {error}"),
            Self::NotFound => formatter.write_str("plan not found"),
            Self::Closed => formatter.write_str("closed plan is read-only"),
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
            Self::NotFound | Self::Closed | Self::InvalidValue { .. } => None,
        }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}
