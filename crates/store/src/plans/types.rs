use std::fmt;

use calc::{ActualOutcome, AssetAllocation, ForecastMode, OutcomeMetrics, Plan, QuickEstimate};

use crate::users::UserId;

pub type PlanId = i64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanSummary {
    pub id: PlanId,
    pub name: String,
    pub season_year: Option<i32>,
    pub note: String,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredPlan {
    pub id: PlanId,
    pub owner_id: UserId,
    pub season_year: Option<i32>,
    pub note: String,
    pub closed: bool,
    pub forecast_mode: ForecastMode,
    pub quick_estimate: QuickEstimate,
    pub starting_capital: Option<rust_decimal::Decimal>,
    pub asset_allocations: Vec<AssetAllocation>,
    pub actual_outcome: Option<StoredActualOutcome>,
    pub plan: Plan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredActualOutcome {
    pub outcome: ActualOutcome,
    pub finalized: bool,
    pub forecast_mode: Option<ForecastMode>,
    pub forecast: Option<OutcomeMetrics>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredSeasonHistory {
    pub id: PlanId,
    pub name: String,
    pub season_year: Option<i32>,
    pub actual_outcome: Option<StoredActualOutcome>,
}

#[derive(Debug)]
pub enum StoreError {
    Database(sqlx::Error),
    NotFound,
    Closed,
    InvalidEmail,
    InvalidPassword,
    DuplicateEmail,
    DuplicateSeasonYear,
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
            Self::DuplicateSeasonYear => {
                formatter.write_str("a season already exists for this owner and year")
            }
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
            | Self::DuplicateSeasonYear
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
