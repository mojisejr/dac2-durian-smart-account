use calc::{
    ActualOutcome, FixedCostLine, Grade, HealthAnswer, KpiTargets, MarketPlan, OutcomeMetrics,
    Plan, ProductionPlan, QuickEstimate, VariableCostLine,
};
use rust_decimal::Decimal;
use sqlx::{PgConnection, Row};

use crate::users::UserId;

use super::{PlanId, StoreError, StoredActualOutcome, StoredPlan, codec};

pub async fn load(
    connection: &mut PgConnection,
    owner_id: UserId,
    plan_id: PlanId,
) -> Result<Option<StoredPlan>, StoreError> {
    let Some(plan_row) = sqlx::query(
        "SELECT name, season_year, note, closed_at IS NOT NULL AS closed,
                starting_capital,
                forecast_mode, quick_sellable_yield_kg,
                quick_average_price_per_kg, quick_total_cost
         FROM plans
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
    .fetch_one(&mut *connection)
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

    let actual_outcome = sqlx::query(
        "SELECT sellable_yield_kg, revenue, total_cost, note,
                finalized_at IS NOT NULL AS finalized, forecast_mode,
                forecast_sellable_yield_kg, forecast_revenue,
                forecast_total_cost, forecast_profit,
                forecast_average_price_per_kg, forecast_cost_per_kg
         FROM season_actual_outcomes
         WHERE plan_id = $1 AND owner_id = $2",
    )
    .bind(plan_id)
    .bind(owner_id)
    .fetch_optional(&mut *connection)
    .await?
    .map(|row| {
        let finalized: bool = row.try_get("finalized")?;
        let forecast_mode = row
            .try_get::<Option<String>, _>("forecast_mode")?
            .map(codec::parse_forecast_mode)
            .transpose()?;
        let forecast = finalized
            .then(|| {
                Ok::<OutcomeMetrics, StoreError>(OutcomeMetrics {
                    sellable_yield_kg: row.try_get("forecast_sellable_yield_kg")?,
                    revenue: row.try_get("forecast_revenue")?,
                    total_cost: row.try_get("forecast_total_cost")?,
                    profit: row.try_get("forecast_profit")?,
                    average_price_per_kg: row.try_get("forecast_average_price_per_kg")?,
                    cost_per_kg: row.try_get("forecast_cost_per_kg")?,
                })
            })
            .transpose()?;
        Ok::<_, StoreError>(StoredActualOutcome {
            outcome: ActualOutcome {
                sellable_yield_kg: row.try_get("sellable_yield_kg")?,
                revenue: row.try_get("revenue")?,
                total_cost: row.try_get("total_cost")?,
                note: row.try_get("note")?,
            },
            finalized,
            forecast_mode,
            forecast,
        })
    })
    .transpose()?;

    let season_year = plan_row.try_get("season_year")?;
    let closed = plan_row.try_get("closed")?;
    let asset_allocations =
        crate::assets::allocations_for_plan(connection, owner_id, plan_id, season_year, closed)
            .await?;

    Ok(Some(StoredPlan {
        id: plan_id,
        owner_id,
        season_year,
        note: plan_row.try_get("note")?,
        closed,
        forecast_mode: codec::parse_forecast_mode(plan_row.try_get("forecast_mode")?)?,
        quick_estimate: QuickEstimate {
            sellable_yield_kg: plan_row.try_get("quick_sellable_yield_kg")?,
            average_price_per_kg: plan_row.try_get("quick_average_price_per_kg")?,
            total_cost: plan_row.try_get("quick_total_cost")?,
        },
        starting_capital: plan_row.try_get("starting_capital")?,
        asset_allocations,
        actual_outcome,
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
