use calc::{AssetAllocation, AssetFacts, AssetKind};
use rust_decimal::Decimal;
use sqlx::{PgConnection, PgPool, Row};

use crate::{
    plans::{PlanId, StoreError},
    users::UserId,
};

pub type AssetId = i64;

#[derive(Clone, Debug, PartialEq)]
pub struct OwnerAsset {
    pub id: AssetId,
    pub owner_id: UserId,
    pub facts: AssetFacts,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetChoice {
    pub asset: OwnerAsset,
    pub selected: bool,
    pub allocation: Option<AssetAllocation>,
}

pub async fn create(
    pool: &PgPool,
    owner_id: UserId,
    facts: &AssetFacts,
) -> Result<OwnerAsset, StoreError> {
    calc::validate(facts).map_err(asset_rule_error)?;
    let id: AssetId = sqlx::query_scalar(
        "INSERT INTO owner_assets (
            owner_id, name, kind, original_cost, start_year,
            useful_life_years, residual_value, retired_year
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id",
    )
    .bind(owner_id)
    .bind(facts.name.trim())
    .bind(asset_kind(facts.kind))
    .bind(facts.original_cost)
    .bind(facts.start_year)
    .bind(facts.useful_life_years.map(|value| value as i32))
    .bind(facts.residual_value)
    .bind(facts.retired_year)
    .fetch_one(pool)
    .await?;
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn update(
    pool: &PgPool,
    owner_id: UserId,
    id: AssetId,
    facts: &AssetFacts,
) -> Result<OwnerAsset, StoreError> {
    calc::validate(facts).map_err(asset_rule_error)?;
    let updated = sqlx::query(
        "UPDATE owner_assets SET
            name = $3, kind = $4, original_cost = $5, start_year = $6,
            useful_life_years = $7, residual_value = $8, retired_year = $9
         WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .bind(facts.name.trim())
    .bind(asset_kind(facts.kind))
    .bind(facts.original_cost)
    .bind(facts.start_year)
    .bind(facts.useful_life_years.map(|value| value as i32))
    .bind(facts.residual_value)
    .bind(facts.retired_year)
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(StoreError::NotFound);
    }
    load(pool, owner_id, id).await?.ok_or(StoreError::NotFound)
}

pub async fn load(
    pool: &PgPool,
    owner_id: UserId,
    id: AssetId,
) -> Result<Option<OwnerAsset>, StoreError> {
    let row = sqlx::query(
        "SELECT name, kind, original_cost, start_year, useful_life_years,
                residual_value, retired_year
         FROM owner_assets WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| owner_asset(id, owner_id, &row)).transpose()
}

pub async fn choices_for_plan(
    pool: &PgPool,
    owner_id: UserId,
    plan_id: PlanId,
) -> Result<Vec<AssetChoice>, StoreError> {
    let plan = sqlx::query(
        "SELECT season_year, closed_at IS NOT NULL AS closed
         FROM plans WHERE id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?;
    let Some(plan) = plan else {
        return Err(StoreError::NotFound);
    };
    if plan.try_get::<bool, _>("closed")? {
        return Err(StoreError::Closed);
    }
    let season_year: i32 = plan
        .try_get::<Option<i32>, _>("season_year")?
        .ok_or_else(|| StoreError::InvalidValue {
            field: "plans.season_year",
            value: "missing".into(),
        })?;
    let rows = sqlx::query(
        "SELECT a.id, a.name, a.kind, a.original_cost, a.start_year,
                a.useful_life_years, a.residual_value, a.retired_year,
                s.asset_id IS NOT NULL AS selected
         FROM owner_assets a
         LEFT JOIN season_asset_selections s
           ON s.asset_id = a.id AND s.owner_id = a.owner_id AND s.plan_id = $2
         WHERE a.owner_id = $1
         ORDER BY a.id",
    )
    .bind(owner_id)
    .bind(plan_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let id: AssetId = row.try_get("id")?;
            let asset = owner_asset(id, owner_id, &row)?;
            let allocation = calc::allocate(&asset.facts, season_year).map_err(asset_rule_error)?;
            let selected = row.try_get::<bool, _>("selected")?;
            Ok(AssetChoice {
                asset,
                selected,
                allocation,
            })
        })
        .collect()
}

pub async fn set_selected(
    pool: &PgPool,
    owner_id: UserId,
    plan_id: PlanId,
    asset_id: AssetId,
    selected: bool,
) -> Result<(), StoreError> {
    let mut transaction = pool.begin().await?;
    let plan = sqlx::query(
        "SELECT season_year, closed_at IS NOT NULL AS closed
         FROM plans WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_optional(transaction.as_mut())
    .await?;
    let Some(plan) = plan else {
        return Err(StoreError::NotFound);
    };
    if plan.try_get::<bool, _>("closed")? {
        return Err(StoreError::Closed);
    }
    let asset = sqlx::query(
        "SELECT name, kind, original_cost, start_year, useful_life_years,
                residual_value, retired_year
         FROM owner_assets WHERE id = $1 AND owner_id = $2",
    )
    .bind(asset_id)
    .bind(owner_id)
    .fetch_optional(transaction.as_mut())
    .await?;
    let Some(asset) = asset else {
        return Err(StoreError::NotFound);
    };
    if selected {
        let season_year = plan
            .try_get::<Option<i32>, _>("season_year")?
            .ok_or_else(|| StoreError::InvalidValue {
                field: "plans.season_year",
                value: "missing".into(),
            })?;
        let facts = owner_asset(asset_id, owner_id, &asset)?.facts;
        if calc::allocate(&facts, season_year)
            .map_err(asset_rule_error)?
            .is_none()
        {
            return Err(StoreError::InvalidValue {
                field: "owner_assets.active_year",
                value: season_year.to_string(),
            });
        }
        sqlx::query(
            "INSERT INTO season_asset_selections (plan_id, owner_id, asset_id)
             VALUES ($1, $2, $3) ON CONFLICT (plan_id, asset_id) DO NOTHING",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(asset_id)
        .execute(transaction.as_mut())
        .await?;
    } else {
        sqlx::query(
            "DELETE FROM season_asset_selections
             WHERE plan_id = $1 AND owner_id = $2 AND asset_id = $3",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(asset_id)
        .execute(transaction.as_mut())
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn allocations_for_plan(
    connection: &mut PgConnection,
    owner_id: UserId,
    plan_id: PlanId,
    season_year: Option<i32>,
    closed: bool,
) -> Result<Vec<AssetAllocation>, StoreError> {
    if closed {
        let rows = sqlx::query(
            "SELECT name, kind, original_cost, start_year, useful_life_years,
                    residual_value, retired_year, annual_depreciation,
                    residual_assumed_zero
             FROM season_asset_snapshots
             WHERE plan_id = $1 AND owner_id = $2 ORDER BY position",
        )
        .bind(plan_id)
        .bind(owner_id)
        .fetch_all(connection)
        .await?;
        return rows
            .into_iter()
            .map(|row| {
                let facts = asset_facts(&row)?;
                Ok(AssetAllocation {
                    investment_value: facts.original_cost,
                    season_year: season_year.unwrap_or(facts.start_year),
                    facts,
                    annual_depreciation: row.try_get("annual_depreciation")?,
                    residual_assumed_zero: row.try_get("residual_assumed_zero")?,
                })
            })
            .collect();
    }

    let Some(season_year) = season_year else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        "SELECT a.name, a.kind, a.original_cost, a.start_year,
                a.useful_life_years, a.residual_value, a.retired_year
         FROM season_asset_selections s
         JOIN owner_assets a ON a.id = s.asset_id AND a.owner_id = s.owner_id
         WHERE s.plan_id = $1 AND s.owner_id = $2
         ORDER BY a.id",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(connection)
    .await?;
    rows.into_iter()
        .filter_map(|row| match asset_facts(&row) {
            Ok(facts) => match calc::allocate(&facts, season_year) {
                Ok(Some(allocation)) => Some(Ok(allocation)),
                Ok(None) => None,
                Err(error) => Some(Err(asset_rule_error(error))),
            },
            Err(error) => Some(Err(error)),
        })
        .collect()
}

pub(crate) async fn snapshot_selected(
    connection: &mut PgConnection,
    owner_id: UserId,
    plan_id: PlanId,
) -> Result<(), StoreError> {
    let season_year: i32 =
        sqlx::query_scalar("SELECT season_year FROM plans WHERE id = $1 AND owner_id = $2")
            .bind(plan_id)
            .bind(owner_id)
            .fetch_one(&mut *connection)
            .await?;
    let rows = sqlx::query(
        "SELECT a.id, a.name, a.kind, a.original_cost, a.start_year,
                a.useful_life_years, a.residual_value, a.retired_year
         FROM season_asset_selections s
         JOIN owner_assets a ON a.id = s.asset_id AND a.owner_id = s.owner_id
         WHERE s.plan_id = $1 AND s.owner_id = $2
         ORDER BY a.id",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(&mut *connection)
    .await?;

    let mut position = 0_i32;
    for row in rows {
        let source_asset_id: AssetId = row.try_get("id")?;
        let facts = asset_facts(&row)?;
        let Some(allocation) = calc::allocate(&facts, season_year).map_err(asset_rule_error)?
        else {
            continue;
        };
        sqlx::query(
            "INSERT INTO season_asset_snapshots (
                plan_id, owner_id, position, source_asset_id, name, kind,
                original_cost, start_year, useful_life_years, residual_value,
                retired_year, annual_depreciation, residual_assumed_zero
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(position)
        .bind(source_asset_id)
        .bind(&allocation.facts.name)
        .bind(asset_kind(allocation.facts.kind))
        .bind(allocation.facts.original_cost)
        .bind(allocation.facts.start_year)
        .bind(allocation.facts.useful_life_years.map(|value| value as i32))
        .bind(allocation.facts.residual_value)
        .bind(allocation.facts.retired_year)
        .bind(allocation.annual_depreciation)
        .bind(allocation.residual_assumed_zero)
        .execute(&mut *connection)
        .await?;
        position += 1;
    }
    Ok(())
}

fn owner_asset(
    id: AssetId,
    owner_id: UserId,
    row: &sqlx::postgres::PgRow,
) -> Result<OwnerAsset, StoreError> {
    Ok(OwnerAsset {
        id,
        owner_id,
        facts: asset_facts(row)?,
    })
}

fn asset_facts(row: &sqlx::postgres::PgRow) -> Result<AssetFacts, StoreError> {
    let useful_life_years = row
        .try_get::<Option<i32>, _>("useful_life_years")?
        .map(|value| {
            u32::try_from(value).map_err(|_| StoreError::InvalidValue {
                field: "owner_assets.useful_life_years",
                value: value.to_string(),
            })
        })
        .transpose()?;
    Ok(AssetFacts {
        name: row.try_get("name")?,
        kind: parse_asset_kind(row.try_get("kind")?)?,
        original_cost: row.try_get::<Decimal, _>("original_cost")?,
        start_year: row.try_get("start_year")?,
        useful_life_years,
        residual_value: row.try_get("residual_value")?,
        retired_year: row.try_get("retired_year")?,
    })
}

fn asset_kind(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Equipment => "equipment",
        AssetKind::OwnedLand => "owned_land",
    }
}

fn parse_asset_kind(value: &str) -> Result<AssetKind, StoreError> {
    match value {
        "equipment" => Ok(AssetKind::Equipment),
        "owned_land" => Ok(AssetKind::OwnedLand),
        _ => Err(StoreError::InvalidValue {
            field: "owner_assets.kind",
            value: value.into(),
        }),
    }
}

fn asset_rule_error(error: calc::AssetRuleError) -> StoreError {
    StoreError::InvalidValue {
        field: "owner_assets",
        value: format!("{error:?}"),
    }
}
