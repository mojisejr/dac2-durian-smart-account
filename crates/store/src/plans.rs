mod codec;
mod read;
mod types;
mod write;

pub use types::{PlanId, PlanSummary, StoreError, StoredActualOutcome, StoredPlan};

use calc::{ActualOutcome, ForecastMode, Plan, QuickEstimate};
use sqlx::{PgConnection, PgPool, Row};

use crate::users::UserId;

pub async fn list(pool: &PgPool, owner_id: UserId) -> Result<Vec<PlanSummary>, StoreError> {
    let rows = sqlx::query(
        "SELECT id, name, season_year, note, closed_at IS NOT NULL AS closed FROM plans \
         WHERE owner_id = $1 ORDER BY season_year DESC NULLS LAST, id DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(PlanSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                season_year: row.try_get("season_year")?,
                note: row.try_get("note")?,
                closed: row.try_get("closed")?,
            })
        })
        .collect()
}

pub async fn create(
    pool: &PgPool,
    owner_id: UserId,
    season_year: i32,
    note: &str,
    plan: &Plan,
) -> Result<StoredPlan, StoreError> {
    create_with_mode(
        pool,
        owner_id,
        season_year,
        note,
        plan,
        ForecastMode::Detailed,
        &QuickEstimate::default(),
    )
    .await
}

pub async fn create_quick(
    pool: &PgPool,
    owner_id: UserId,
    season_year: i32,
    note: &str,
    plan: &Plan,
) -> Result<StoredPlan, StoreError> {
    create_with_mode(
        pool,
        owner_id,
        season_year,
        note,
        plan,
        ForecastMode::Quick,
        &QuickEstimate::default(),
    )
    .await
}

async fn create_with_mode(
    pool: &PgPool,
    owner_id: UserId,
    season_year: i32,
    note: &str,
    plan: &Plan,
    forecast_mode: ForecastMode,
    quick_estimate: &QuickEstimate,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    let id: PlanId = sqlx::query_scalar(
        "INSERT INTO plans (
            owner_id, name, season_year, note, forecast_mode,
            quick_sellable_yield_kg, quick_average_price_per_kg, quick_total_cost
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(owner_id)
    .bind(&plan.name)
    .bind(season_year)
    .bind(note.trim())
    .bind(codec::forecast_mode(forecast_mode))
    .bind(quick_estimate.sellable_yield_kg)
    .bind(quick_estimate.average_price_per_kg)
    .bind(quick_estimate.total_cost)
    .fetch_one(transaction.as_mut())
    .await
    .map_err(season_write_error)?;
    write::replace_sections(transaction.as_mut(), owner_id, id, plan).await?;
    transaction.commit().await?;

    Ok(StoredPlan {
        id,
        owner_id,
        season_year: Some(season_year),
        note: note.trim().into(),
        closed: false,
        forecast_mode,
        quick_estimate: quick_estimate.clone(),
        actual_outcome: None,
        plan: plan.clone(),
    })
}

pub async fn load(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
) -> Result<Option<StoredPlan>, StoreError> {
    let mut connection = pool.acquire().await?;
    read::load(&mut connection, owner_id, id).await
}

pub async fn save(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    plan: &Plan,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("UPDATE plans SET name = $3 WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .bind(&plan.name)
        .execute(transaction.as_mut())
        .await?;
    write::replace_sections(transaction.as_mut(), owner_id, id, plan).await?;
    transaction.commit().await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn save_quick(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    estimate: &QuickEstimate,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query(
        "UPDATE plans SET
            forecast_mode = 'quick',
            quick_sellable_yield_kg = $3,
            quick_average_price_per_kg = $4,
            quick_total_cost = $5
         WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .bind(estimate.sellable_yield_kg)
    .bind(estimate.average_price_per_kg)
    .bind(estimate.total_cost)
    .execute(transaction.as_mut())
    .await?;
    transaction.commit().await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn set_forecast_mode(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    mode: ForecastMode,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("UPDATE plans SET forecast_mode = $3 WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .bind(codec::forecast_mode(mode))
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn update_metadata(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    season_year: i32,
    name: &str,
    note: &str,
) -> Result<(), StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query(
        "UPDATE plans SET season_year = $3, name = $4, note = $5 \
         WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .bind(season_year)
    .bind(name.trim())
    .bind(note.trim())
    .execute(transaction.as_mut())
    .await
    .map_err(season_write_error)?;
    transaction.commit().await?;
    Ok(())
}

pub async fn save_actual_draft(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    outcome: &ActualOutcome,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query(
        "INSERT INTO season_actual_outcomes (
            plan_id, owner_id, sellable_yield_kg, revenue, total_cost, note
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (plan_id) DO UPDATE SET
            sellable_yield_kg = EXCLUDED.sellable_yield_kg,
            revenue = EXCLUDED.revenue,
            total_cost = EXCLUDED.total_cost,
            note = EXCLUDED.note",
    )
    .bind(id)
    .bind(owner_id)
    .bind(outcome.sellable_yield_kg)
    .bind(outcome.revenue)
    .bind(outcome.total_cost)
    .bind(outcome.note.trim())
    .execute(transaction.as_mut())
    .await?;
    transaction.commit().await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn finalize_with_actual(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    outcome: &ActualOutcome,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        "SELECT closed_at IS NOT NULL AS closed FROM plans
         WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(transaction.as_mut())
    .await?;
    let Some(row) = row else {
        return Err(StoreError::NotFound);
    };

    if row.try_get::<bool, _>("closed")? {
        let existing = read::load(transaction.as_mut(), owner_id, id)
            .await?
            .ok_or(StoreError::NotFound)?;
        if existing
            .actual_outcome
            .as_ref()
            .is_some_and(|actual| actual.finalized && actual.outcome == normalized(outcome))
        {
            transaction.commit().await?;
            return Ok(existing);
        }
        return Err(StoreError::Closed);
    }

    if !calc::analyze_actual(outcome).input_issues.is_empty() {
        return Err(StoreError::InvalidValue {
            field: "season_actual_outcomes",
            value: "incomplete or negative actual facts".into(),
        });
    }

    let stored = read::load(transaction.as_mut(), owner_id, id)
        .await?
        .ok_or(StoreError::NotFound)?;
    let forecast =
        calc::forecast_metrics(stored.forecast_mode, &stored.quick_estimate, &stored.plan);
    let outcome = normalized(outcome);
    sqlx::query(
        "INSERT INTO season_actual_outcomes (
            plan_id, owner_id, sellable_yield_kg, revenue, total_cost, note,
            finalized_at, forecast_mode, forecast_sellable_yield_kg,
            forecast_revenue, forecast_total_cost, forecast_profit,
            forecast_average_price_per_kg, forecast_cost_per_kg
         ) VALUES (
            $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, $7, $8, $9, $10,
            $11, $12, $13
         )
         ON CONFLICT (plan_id) DO UPDATE SET
            sellable_yield_kg = EXCLUDED.sellable_yield_kg,
            revenue = EXCLUDED.revenue,
            total_cost = EXCLUDED.total_cost,
            note = EXCLUDED.note,
            finalized_at = EXCLUDED.finalized_at,
            forecast_mode = EXCLUDED.forecast_mode,
            forecast_sellable_yield_kg = EXCLUDED.forecast_sellable_yield_kg,
            forecast_revenue = EXCLUDED.forecast_revenue,
            forecast_total_cost = EXCLUDED.forecast_total_cost,
            forecast_profit = EXCLUDED.forecast_profit,
            forecast_average_price_per_kg = EXCLUDED.forecast_average_price_per_kg,
            forecast_cost_per_kg = EXCLUDED.forecast_cost_per_kg",
    )
    .bind(id)
    .bind(owner_id)
    .bind(outcome.sellable_yield_kg)
    .bind(outcome.revenue)
    .bind(outcome.total_cost)
    .bind(&outcome.note)
    .bind(codec::forecast_mode(stored.forecast_mode))
    .bind(forecast.sellable_yield_kg)
    .bind(forecast.revenue)
    .bind(forecast.total_cost)
    .bind(forecast.profit)
    .bind(forecast.average_price_per_kg)
    .bind(forecast.cost_per_kg)
    .execute(transaction.as_mut())
    .await?;
    sqlx::query("UPDATE plans SET closed_at = CURRENT_TIMESTAMP WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn delete(pool: &PgPool, owner_id: UserId, id: PlanId) -> Result<(), StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("DELETE FROM plans WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    Ok(())
}

/// Deep-copy a source plan without mutating it. A closed season may therefore
/// be used as the source of a new open season while remaining read-only itself.
pub async fn duplicate(
    pool: &PgPool,
    owner_id: UserId,
    source_id: PlanId,
    season_year: i32,
    new_name: &str,
    note: &str,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    let Some(mut source) = read::load(transaction.as_mut(), owner_id, source_id).await? else {
        return Err(StoreError::NotFound);
    };
    source.plan.name = new_name.into();
    let forecast_mode = ForecastMode::Quick;
    let id: PlanId = sqlx::query_scalar(
        "INSERT INTO plans (
            owner_id, name, season_year, note, forecast_mode,
            quick_sellable_yield_kg, quick_average_price_per_kg, quick_total_cost
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(owner_id)
    .bind(&source.plan.name)
    .bind(season_year)
    .bind(note.trim())
    .bind(codec::forecast_mode(forecast_mode))
    .bind(source.quick_estimate.sellable_yield_kg)
    .bind(source.quick_estimate.average_price_per_kg)
    .bind(source.quick_estimate.total_cost)
    .fetch_one(transaction.as_mut())
    .await
    .map_err(season_write_error)?;
    write::replace_sections(transaction.as_mut(), owner_id, id, &source.plan).await?;
    transaction.commit().await?;

    Ok(StoredPlan {
        id,
        owner_id,
        season_year: Some(season_year),
        note: note.trim().into(),
        closed: false,
        forecast_mode,
        quick_estimate: source.quick_estimate,
        actual_outcome: None,
        plan: source.plan,
    })
}

fn normalized(outcome: &ActualOutcome) -> ActualOutcome {
    ActualOutcome {
        sellable_yield_kg: outcome.sellable_yield_kg,
        revenue: outcome.revenue,
        total_cost: outcome.total_cost,
        note: outcome.note.trim().into(),
    }
}

fn season_write_error(error: sqlx::Error) -> StoreError {
    if let sqlx::Error::Database(database) = &error
        && database.constraint() == Some("plans_owner_season_year_unique")
    {
        return StoreError::DuplicateSeasonYear;
    }
    StoreError::Database(error)
}

async fn ensure_open(
    connection: &mut PgConnection,
    owner_id: UserId,
    id: PlanId,
) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT closed_at IS NOT NULL AS closed FROM plans \
         WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(connection)
    .await?;
    let Some(row) = row else {
        return Err(StoreError::NotFound);
    };
    if row.try_get::<bool, _>("closed")? {
        return Err(StoreError::Closed);
    }
    Ok(())
}
