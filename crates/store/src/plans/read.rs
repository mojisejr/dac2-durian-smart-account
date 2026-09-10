use calc::{
    FixedCostLine, Grade, HealthAnswer, KpiTargets, MarketPlan, Plan, ProductionPlan,
    VariableCostLine,
};
use rust_decimal::Decimal;
use sqlx::{PgConnection, Row};

use crate::users::UserId;

use super::{PlanId, StoreError, StoredPlan, codec};

pub async fn load(
    connection: &mut PgConnection,
    owner_id: UserId,
    plan_id: PlanId,
) -> Result<Option<StoredPlan>, StoreError> {
    let Some(plan_row) = sqlx::query(
        "SELECT name, season_year, note, closed_at IS NOT NULL AS closed FROM plans
         WHERE id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_optional(&mut *connection)
    .await?
    else {
        return Ok(None);
    };

    let market = sqlx::query(
        "SELECT target_customer, demand_kg, minimum_price_per_kg, sales_period,
                sales_channels, largest_buyer_share, quality_requirements
         FROM market_plans WHERE plan_id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_one(&mut *connection)
    .await?;
    let sales_channels = market
        .try_get::<Option<i64>, _>("sales_channels")?
        .map(|value| {
            u32::try_from(value).map_err(|_| StoreError::InvalidValue {
                field: "market_plans.sales_channels",
                value: value.to_string(),
            })
        })
        .transpose()?;
    let market = MarketPlan {
        target_customer: market.try_get("target_customer")?,
        demand_kg: market.try_get::<Option<Decimal>, _>("demand_kg")?,
        minimum_price_per_kg: market.try_get("minimum_price_per_kg")?,
        sales_period: market.try_get("sales_period")?,
        sales_channels,
        largest_buyer_share: market.try_get("largest_buyer_share")?,
        quality_requirements: market.try_get("quality_requirements")?,
    };

    let production = sqlx::query(
        "SELECT area_rai, producing_trees, fruits_per_tree,
                average_fruit_weight_kg, loss_share
         FROM yield_estimates WHERE plan_id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_one(&mut *connection)
    .await?;
    let mut production = ProductionPlan {
        area_rai: production.try_get("area_rai")?,
        producing_trees: production.try_get("producing_trees")?,
        fruits_per_tree: production.try_get("fruits_per_tree")?,
        average_fruit_weight_kg: production.try_get("average_fruit_weight_kg")?,
        loss_share: production.try_get("loss_share")?,
        grades: Vec::new(),
    };

    let rows = sqlx::query(
        "SELECT name, share, price_per_kg, counts_as_quality_grade
         FROM grade_mix WHERE plan_id = $1 AND owner_id = $2 ORDER BY position",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(&mut *connection)
    .await?;
    production.grades = rows
        .into_iter()
        .map(|row| {
            Ok(Grade {
                name: row.try_get("name")?,
                share: row.try_get("share")?,
                price_per_kg: row.try_get("price_per_kg")?,
                counts_as_quality_grade: row.try_get("counts_as_quality_grade")?,
            })
        })
        .collect::<Result<_, StoreError>>()?;

    let rows = sqlx::query(
        "SELECT name, kind, quantity, unit, unit_price
         FROM variable_cost_lines
         WHERE plan_id = $1 AND owner_id = $2 ORDER BY position",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(&mut *connection)
    .await?;
    let variable_costs = rows
        .into_iter()
        .map(|row| {
            Ok(VariableCostLine {
                name: row.try_get("name")?,
                kind: codec::parse_variable_cost_kind(row.try_get("kind")?)?,
                quantity: row.try_get("quantity")?,
                unit: row.try_get("unit")?,
                unit_price: row.try_get("unit_price")?,
            })
        })
        .collect::<Result<_, StoreError>>()?;

    let rows = sqlx::query(
        "SELECT name, cash_kind, amount_per_year, investment_base
         FROM fixed_cost_lines
         WHERE plan_id = $1 AND owner_id = $2 ORDER BY position",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(&mut *connection)
    .await?;
    let fixed_costs = rows
        .into_iter()
        .map(|row| {
            Ok(FixedCostLine {
                name: row.try_get("name")?,
                cash_kind: codec::parse_cash_kind(row.try_get("cash_kind")?)?,
                amount_per_year: row.try_get("amount_per_year")?,
                investment_base: row.try_get("investment_base")?,
            })
        })
        .collect::<Result<_, StoreError>>()?;

    let rows = sqlx::query(
        "SELECT question, score FROM health_answers
         WHERE plan_id = $1 AND owner_id = $2 ORDER BY position",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_all(&mut *connection)
    .await?;
    let health_answers = rows
        .into_iter()
        .map(|row| {
            let score = row
                .try_get::<Option<i16>, _>("score")?
                .map(|value| {
                    u8::try_from(value).map_err(|_| StoreError::InvalidValue {
                        field: "health_answers.score",
                        value: value.to_string(),
                    })
                })
                .transpose()?;
            Ok(HealthAnswer {
                question: codec::parse_health_question(row.try_get("question")?)?,
                score,
            })
        })
        .collect::<Result<_, StoreError>>()?;

    let targets = sqlx::query(
        "SELECT yield_per_rai, yield_per_tree, yield_per_labor_day,
                yield_per_fertilizer_kg, yield_per_water_cubic_meter,
                yield_per_kwh, quality_grade_share, loss_share, cost_per_kg
         FROM kpi_targets WHERE plan_id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_one(connection)
    .await?;
    let targets = KpiTargets {
        yield_per_rai: targets.try_get("yield_per_rai")?,
        yield_per_tree: targets.try_get("yield_per_tree")?,
        yield_per_labor_day: targets.try_get("yield_per_labor_day")?,
        yield_per_fertilizer_kg: targets.try_get("yield_per_fertilizer_kg")?,
        yield_per_water_cubic_meter: targets.try_get("yield_per_water_cubic_meter")?,
        yield_per_kwh: targets.try_get("yield_per_kwh")?,
        quality_grade_share: targets.try_get("quality_grade_share")?,
        loss_share: targets.try_get("loss_share")?,
        cost_per_kg: targets.try_get("cost_per_kg")?,
    };

    Ok(Some(StoredPlan {
        id: plan_id,
        owner_id,
        season_year: plan_row.try_get("season_year")?,
        note: plan_row.try_get("note")?,
        closed: plan_row.try_get("closed")?,
        plan: Plan {
            name: plan_row.try_get("name")?,
            market,
            production,
            variable_costs,
            fixed_costs,
            health_answers,
            targets,
        },
    }))
}
