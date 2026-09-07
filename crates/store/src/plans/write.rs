use calc::Plan;
use sqlx::PgConnection;

use crate::users::UserId;

use super::{PlanId, StoreError, codec};

pub async fn replace_sections(
    connection: &mut PgConnection,
    owner_id: UserId,
    plan_id: PlanId,
    plan: &Plan,
) -> Result<(), StoreError> {
    for statement in [
        "DELETE FROM market_plans WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM yield_estimates WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM grade_mix WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM variable_cost_lines WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM fixed_cost_lines WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM health_answers WHERE plan_id = $1 AND owner_id = $2",
        "DELETE FROM kpi_targets WHERE plan_id = $1 AND owner_id = $2",
    ] {
        sqlx::query(statement)
            .bind(plan_id)
            .bind(owner_id)
            .execute(&mut *connection)
            .await?;
    }

    sqlx::query(
        "INSERT INTO market_plans (
            plan_id, owner_id, target_customer, demand_kg, minimum_price_per_kg,
            sales_period, sales_channels, largest_buyer_share, quality_requirements
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(plan_id)
    .bind(owner_id)
    .bind(plan.market.target_customer.as_deref())
    .bind(plan.market.demand_kg)
    .bind(plan.market.minimum_price_per_kg)
    .bind(plan.market.sales_period.as_deref())
    .bind(plan.market.sales_channels.map(i64::from))
    .bind(plan.market.largest_buyer_share)
    .bind(plan.market.quality_requirements.as_deref())
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "INSERT INTO yield_estimates (
            plan_id, owner_id, area_rai, producing_trees, fruits_per_tree,
            average_fruit_weight_kg, loss_share
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(plan_id)
    .bind(owner_id)
    .bind(plan.production.area_rai)
    .bind(plan.production.producing_trees)
    .bind(plan.production.fruits_per_tree)
    .bind(plan.production.average_fruit_weight_kg)
    .bind(plan.production.loss_share)
    .execute(&mut *connection)
    .await?;

    for (index, grade) in plan.production.grades.iter().enumerate() {
        sqlx::query(
            "INSERT INTO grade_mix (
                plan_id, owner_id, position, name, share, price_per_kg,
                counts_as_quality_grade
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(position(index)?)
        .bind(&grade.name)
        .bind(grade.share)
        .bind(grade.price_per_kg)
        .bind(grade.counts_as_quality_grade)
        .execute(&mut *connection)
        .await?;
    }

    for (index, line) in plan.variable_costs.iter().enumerate() {
        sqlx::query(
            "INSERT INTO variable_cost_lines (
                plan_id, owner_id, position, name, kind, quantity, unit, unit_price
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(position(index)?)
        .bind(&line.name)
        .bind(codec::variable_cost_kind(line.kind))
        .bind(line.quantity)
        .bind(&line.unit)
        .bind(line.unit_price)
        .execute(&mut *connection)
        .await?;
    }

    for (index, line) in plan.fixed_costs.iter().enumerate() {
        sqlx::query(
            "INSERT INTO fixed_cost_lines (
                plan_id, owner_id, position, name, cash_kind, amount_per_year,
                investment_base
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(position(index)?)
        .bind(&line.name)
        .bind(codec::cash_kind(line.cash_kind))
        .bind(line.amount_per_year)
        .bind(line.investment_base)
        .execute(&mut *connection)
        .await?;
    }

    for (index, answer) in plan.health_answers.iter().enumerate() {
        sqlx::query(
            "INSERT INTO health_answers (
                plan_id, owner_id, position, question, score
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(plan_id)
        .bind(owner_id)
        .bind(position(index)?)
        .bind(codec::health_question(answer.question))
        .bind(answer.score.map(i16::from))
        .execute(&mut *connection)
        .await?;
    }

    let targets = &plan.targets;
    sqlx::query(
        "INSERT INTO kpi_targets (
            plan_id, owner_id, yield_per_rai, yield_per_tree, yield_per_labor_day,
            yield_per_fertilizer_kg, yield_per_water_cubic_meter, yield_per_kwh,
            quality_grade_share, loss_share, cost_per_kg
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(plan_id)
    .bind(owner_id)
    .bind(targets.yield_per_rai)
    .bind(targets.yield_per_tree)
    .bind(targets.yield_per_labor_day)
    .bind(targets.yield_per_fertilizer_kg)
    .bind(targets.yield_per_water_cubic_meter)
    .bind(targets.yield_per_kwh)
    .bind(targets.quality_grade_share)
    .bind(targets.loss_share)
    .bind(targets.cost_per_kg)
    .execute(connection)
    .await?;

    Ok(())
}

fn position(index: usize) -> Result<i32, StoreError> {
    i32::try_from(index).map_err(|_| StoreError::InvalidValue {
        field: "position",
        value: index.to_string(),
    })
}
