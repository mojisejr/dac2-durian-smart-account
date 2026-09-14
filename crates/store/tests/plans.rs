use std::borrow::Cow;
use std::path::Path;

use calc::{
    ActualOutcome, CostSectionState, ForecastMode, Grade, PriceSource, QuickEstimate,
    UnclassifiedExpense, VariableCostKind, VariableCostLine, YieldSource, workbook_sample,
};
use rust_decimal::Decimal;
use sqlx::migrate::Migrator;
use sqlx::{PgPool, Row};
use store::plans::{self, StoreError};
use store::users;

fn ten_grade_plan() -> calc::Plan {
    let mut plan = workbook_sample();
    plan.production.grades = (1..=10)
        .map(|index| Grade {
            name: format!("เกรด {index}"),
            share: Some(Decimal::new(1, 1)),
            price_per_kg: Some(Decimal::from(index * 10)),
            counts_as_quality_grade: index <= 2,
        })
        .collect();
    plan
}

fn assert_not_found<T>(result: Result<T, StoreError>) {
    assert!(matches!(result, Err(StoreError::NotFound)));
}

fn assert_closed<T>(result: Result<T, StoreError>) {
    assert!(matches!(result, Err(StoreError::Closed)));
}

fn actual_outcome() -> ActualOutcome {
    ActualOutcome {
        sellable_yield_kg: Some(Decimal::from(18_000)),
        revenue: Some(Decimal::from(1_530_000)),
        total_cost: Some(Decimal::from(990_000)),
        note: "ผลผลิตน้อยกว่าคาด".into(),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn complete_ten_grade_plan_round_trips_without_loss(pool: PgPool) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let plan = ten_grade_plan();

    let created = plans::create(&pool, owner.id, 2569, "ฤดูทดสอบ", &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("owner can load the plan");

    assert_eq!(loaded.owner_id, owner.id);
    assert!(!loaded.closed);
    assert_eq!(loaded.season_year, Some(2569));
    assert_eq!(loaded.note, "ฤดูทดสอบ");
    assert_eq!(loaded.plan, plan);
    assert_eq!(loaded.plan.production.grades.len(), 10);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_empty_in_progress_plan_round_trips_as_missing_not_zero(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let plan = calc::Plan::default();

    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("owner can load the in-progress plan");

    assert_eq!(loaded.plan, plan);
    assert!(loaded.plan.market.buyer_committed_kg.is_none());
    assert!(loaded.plan.production.grades.is_empty());
    assert!(loaded.plan.targets.yield_per_rai.is_none());
    assert_eq!(loaded.forecast_mode, ForecastMode::Detailed);
    assert_eq!(loaded.quick_estimate, QuickEstimate::default());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn direct_yield_and_average_price_round_trip_with_both_branches_kept(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let mut plan = workbook_sample();
    plan.production.yield_source = YieldSource::Direct;
    plan.production.sellable_yield_kg = Some(Decimal::from(18_500));
    plan.production.price_source = PriceSource::Average;
    plan.production.average_price_per_kg = Some(Decimal::new(7_925, 2));

    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("owner can load the plan");

    assert_eq!(loaded.plan, plan);
    assert_eq!(loaded.plan.production.yield_source, YieldSource::Direct);
    assert_eq!(loaded.plan.production.price_source, PriceSource::Average);
    assert_eq!(
        loaded.plan.production.producing_trees,
        Some(Decimal::from(200)),
        "the unselected derived facts are kept"
    );
    assert_eq!(
        loaded.plan.production.grades.len(),
        4,
        "the unselected grade list is kept"
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn blank_zero_and_entered_branch_values_round_trip_distinctly(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let mut plan = calc::Plan::default();
    plan.production.yield_source = YieldSource::Direct;
    plan.production.price_source = PriceSource::Average;

    let states = [
        (None, None),
        (Some(Decimal::ZERO), Some(Decimal::ZERO)),
        (Some(Decimal::from(12_000)), Some(Decimal::from(95))),
    ];
    for (index, (yield_kg, price)) in states.into_iter().enumerate() {
        plan.production.sellable_yield_kg = yield_kg;
        plan.production.average_price_per_kg = price;
        plan.market.buyer_committed_kg = yield_kg;
        let year = 2569 + i32::try_from(index).expect("small index");
        let created = plans::create(&pool, owner.id, year, "", &plan).await?;
        let loaded = plans::load(&pool, owner.id, created.id)
            .await?
            .expect("owner can load the plan");
        assert_eq!(loaded.plan.production.sellable_yield_kg, yield_kg);
        assert_eq!(loaded.plan.production.average_price_per_kg, price);
        assert_eq!(loaded.plan.market.buyer_committed_kg, yield_kg);
    }
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn cost_section_states_round_trip_derive_from_rows_and_refuse_a_false_none(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;

    // Unknown is the default for an empty plan.
    let mut plan = calc::Plan::default();
    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert_eq!(loaded.plan.variable_cost_state, CostSectionState::Unknown);
    assert_eq!(loaded.plan.fixed_cost_state, CostSectionState::Unknown);

    // Confirmed none round-trips while the lists are empty.
    plan.variable_cost_state = CostSectionState::ConfirmedNone;
    plan.fixed_cost_state = CostSectionState::ConfirmedNone;
    plans::save(&pool, owner.id, created.id, &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert_eq!(
        loaded.plan.variable_cost_state,
        CostSectionState::ConfirmedNone
    );
    assert_eq!(
        loaded.plan.fixed_cost_state,
        CostSectionState::ConfirmedNone
    );
    assert_eq!(
        calc::analyze(&loaded.plan).cost.total_cost,
        Some(Decimal::ZERO)
    );

    // Rows make the stored state entered_items whatever was claimed.
    plan.variable_cost_state = CostSectionState::Unknown;
    plan.variable_costs = workbook_sample().variable_costs;
    plans::save(&pool, owner.id, created.id, &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert_eq!(
        loaded.plan.variable_cost_state,
        CostSectionState::EnteredItems
    );

    // Removing the last row reverts to unknown, never to confirmed none.
    plan.variable_cost_state = CostSectionState::EnteredItems;
    plan.variable_costs.clear();
    plans::save(&pool, owner.id, created.id, &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert_eq!(loaded.plan.variable_cost_state, CostSectionState::Unknown);

    // Confirmed none with rows is refused, and nothing is written.
    plan.fixed_cost_state = CostSectionState::ConfirmedNone;
    plan.fixed_costs = workbook_sample().fixed_costs;
    assert!(matches!(
        plans::save(&pool, owner.id, created.id, &plan).await,
        Err(StoreError::InvalidValue {
            field: "plans.fixed_cost_state",
            ..
        })
    ));
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert!(loaded.plan.fixed_costs.is_empty());
    assert_eq!(
        loaded.plan.fixed_cost_state,
        CostSectionState::ConfirmedNone
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn unclassified_expenses_and_total_lines_round_trip_duplicate_and_freeze(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let mut plan = workbook_sample();
    plan.unclassified_expenses = vec![
        UnclassifiedExpense {
            name: "ค่าอะไรสักอย่างเดือนสาม".into(),
            amount: Some(Decimal::from(50_000)),
            note: "จำได้ว่าจ่ายให้คนขับรถ".into(),
        },
        UnclassifiedExpense {
            name: "ยังไม่รู้ยอด".into(),
            amount: None,
            note: String::new(),
        },
    ];
    plan.variable_costs.push(VariableCostLine {
        name: "ค่าจ้างเก็บที่จำได้แต่ยอดรวม".into(),
        kind: VariableCostKind::HarvestLabor,
        quantity: None,
        unit: String::new(),
        unit_price: None,
        total_amount: Some(Decimal::from(12_000)),
    });
    let expected_total = calc::analyze(&plan).cost.total_cost.expect("known total");

    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    let loaded = plans::load(&pool, owner.id, created.id).await?.unwrap();
    assert_eq!(loaded.plan, plan);

    let duplicate = plans::duplicate(&pool, owner.id, created.id, 2570, "ปีหน้า", "").await?;
    assert_eq!(
        duplicate.plan.unclassified_expenses,
        plan.unclassified_expenses
    );
    assert_eq!(
        duplicate.plan.variable_costs.last().unwrap().total_amount,
        Some(Decimal::from(12_000))
    );

    let closed =
        plans::finalize_with_actual(&pool, owner.id, created.id, &actual_outcome()).await?;
    let frozen = closed.actual_outcome.unwrap().forecast.unwrap();
    assert_eq!(
        frozen.total_cost,
        Some(expected_total),
        "the frozen total counts the total-only line and ignores unclassified captures"
    );
    let mut reclassified = plan.clone();
    reclassified.unclassified_expenses.clear();
    assert_closed(plans::save(&pool, owner.id, created.id, &reclassified).await);

    // A line with both a total and a unit price is refused by the database
    // as well as by the input contract.
    let mut invalid = plan.clone();
    invalid.variable_costs.last_mut().unwrap().unit_price = Some(Decimal::ONE);
    assert!(matches!(
        plans::save(&pool, owner.id, duplicate.id, &invalid).await,
        Err(StoreError::Database(_))
    ));
    Ok(())
}

const STATES_MIGRATION: i64 = 202609130002;

#[sqlx::test(migrations = false)]
async fn existing_cost_rows_survive_the_states_migration_and_its_rollback(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut migrator = Migrator::new(Path::new("../../migrations")).await?;
    let every_migration = migrator.migrations.to_vec();
    migrator.migrations = Cow::Owned(
        every_migration
            .iter()
            .filter(|migration| migration.version < STATES_MIGRATION)
            .cloned()
            .collect(),
    );
    migrator.run(&pool).await?;

    let owner = users::create(&pool, "owner@example.test").await?;
    let plan_id: i64 = sqlx::query_scalar(
        "INSERT INTO plans (owner_id, name, season_year) VALUES ($1, 'ปีเก่า', 2568)
         RETURNING id",
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await?;
    for statement in [
        "INSERT INTO market_plans (plan_id, owner_id) VALUES ($1, $2)",
        "INSERT INTO yield_estimates (plan_id, owner_id) VALUES ($1, $2)",
        "INSERT INTO kpi_targets (plan_id, owner_id) VALUES ($1, $2)",
        "INSERT INTO variable_cost_lines (plan_id, owner_id, position, name, kind, quantity, unit, unit_price)
         VALUES ($1, $2, 0, 'ปุ๋ย', 'fertilizer', 5000, 'กก.', 20)",
    ] {
        sqlx::query(statement)
            .bind(plan_id)
            .bind(owner.id)
            .execute(&pool)
            .await?;
    }

    migrator.migrations = Cow::Owned(every_migration);
    migrator.run(&pool).await?;

    let loaded = plans::load(&pool, owner.id, plan_id)
        .await?
        .expect("the legacy plan loads");
    assert_eq!(
        loaded.plan.variable_cost_state,
        CostSectionState::Unknown,
        "the stored default is unknown; rows still make the effective state entered items"
    );
    assert_eq!(
        loaded.plan.effective_variable_cost_state(),
        CostSectionState::EnteredItems
    );
    assert_eq!(loaded.plan.fixed_cost_state, CostSectionState::Unknown);
    assert_eq!(loaded.plan.variable_costs[0].total_amount, None);
    assert_eq!(
        calc::analyze(&loaded.plan).cost.variable_cost,
        Some(Decimal::from(100_000)),
        "a legacy line still totals quantity times unit price"
    );
    assert!(loaded.plan.unclassified_expenses.is_empty());

    let rollback = std::fs::read_to_string(
        "../../migrations/rollback/202609130002_cost_knowledge_states.sql",
    )?;
    sqlx::raw_sql(&rollback).execute(&pool).await?;
    let quantity: Decimal = sqlx::query_scalar(
        "SELECT quantity FROM variable_cost_lines WHERE plan_id = $1 AND position = 0",
    )
    .bind(plan_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(quantity, Decimal::from(5_000));
    let dropped: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE (table_name = 'plans' AND column_name IN ('variable_cost_state', 'fixed_cost_state'))
            OR (table_name = 'variable_cost_lines' AND column_name = 'total_amount')
            OR table_name = 'unclassified_expenses'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(dropped, 0, "rollback removes every added column and table");
    Ok(())
}

const BRANCHES_MIGRATION: i64 = 202609130001;

/// Rows written before the Batch 2 migration keep their values through the
/// column rename and the new source columns, and again through the rollback
/// script, so the migration can be stepped in either direction without loss.
#[sqlx::test(migrations = false)]
async fn existing_rows_survive_the_branches_migration_and_its_rollback(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut migrator = Migrator::new(Path::new("../../migrations")).await?;
    let every_migration = migrator.migrations.to_vec();
    migrator.migrations = Cow::Owned(
        every_migration
            .iter()
            .filter(|migration| migration.version < BRANCHES_MIGRATION)
            .cloned()
            .collect(),
    );
    migrator.run(&pool).await?;

    let owner = users::create(&pool, "owner@example.test").await?;
    let plan_id: i64 = sqlx::query_scalar(
        "INSERT INTO plans (owner_id, name, season_year) VALUES ($1, 'ปีเก่า', 2568)
         RETURNING id",
    )
    .bind(owner.id)
    .fetch_one(&pool)
    .await?;
    sqlx::query(
        "INSERT INTO market_plans (plan_id, owner_id, demand_kg, target_customer)
         VALUES ($1, $2, 25000, 'ล้งส่งออก')",
    )
    .bind(plan_id)
    .bind(owner.id)
    .execute(&pool)
    .await?;
    sqlx::query(
        "INSERT INTO yield_estimates (
            plan_id, owner_id, area_rai, producing_trees, fruits_per_tree,
            average_fruit_weight_kg, loss_share
         ) VALUES ($1, $2, 10, 200, 35, 3, 0.05)",
    )
    .bind(plan_id)
    .bind(owner.id)
    .execute(&pool)
    .await?;
    for statement in [
        "INSERT INTO grade_mix (plan_id, owner_id, position, name, share, price_per_kg, counts_as_quality_grade)
         VALUES ($1, $2, 0, 'A', 1, 82.5, TRUE)",
        "INSERT INTO kpi_targets (plan_id, owner_id) VALUES ($1, $2)",
    ] {
        sqlx::query(statement)
            .bind(plan_id)
            .bind(owner.id)
            .execute(&pool)
            .await?;
    }

    migrator.migrations = Cow::Owned(every_migration);
    migrator.run(&pool).await?;

    let migrated = sqlx::query(
        "SELECT m.buyer_committed_kg, m.target_customer, y.yield_source, y.sellable_yield_kg,
                y.price_source, y.average_price_per_kg, y.producing_trees
         FROM market_plans m JOIN yield_estimates y USING (plan_id)
         WHERE m.plan_id = $1",
    )
    .bind(plan_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        migrated.try_get::<Decimal, _>("buyer_committed_kg")?,
        Decimal::from(25_000),
        "the renamed column keeps the stored quantity"
    );
    assert_eq!(migrated.try_get::<String, _>("target_customer")?, "ล้งส่งออก");
    assert_eq!(migrated.try_get::<String, _>("yield_source")?, "derived");
    assert_eq!(migrated.try_get::<String, _>("price_source")?, "by_grade");
    assert_eq!(
        migrated.try_get::<Option<Decimal>, _>("sellable_yield_kg")?,
        None
    );
    assert_eq!(
        migrated.try_get::<Option<Decimal>, _>("average_price_per_kg")?,
        None
    );
    assert_eq!(
        migrated.try_get::<Decimal, _>("producing_trees")?,
        Decimal::from(200)
    );

    let loaded = plans::load(&pool, owner.id, plan_id)
        .await?
        .expect("the legacy plan loads through the current store");
    assert_eq!(
        loaded.plan.market.buyer_committed_kg,
        Some(Decimal::from(25_000))
    );
    assert_eq!(loaded.plan.production.yield_source, YieldSource::Derived);
    assert_eq!(loaded.plan.production.price_source, PriceSource::ByGrade);
    assert_eq!(
        calc::analyze(&loaded.plan).revenue.sellable_yield_kg,
        Some(Decimal::from(19_950)),
        "a legacy row still derives its yield exactly as before"
    );

    let rollback = std::fs::read_to_string(
        "../../migrations/rollback/202609130001_sell_and_harvest_branches.sql",
    )?;
    sqlx::raw_sql(&rollback).execute(&pool).await?;

    let rolled_back =
        sqlx::query("SELECT demand_kg, target_customer FROM market_plans WHERE plan_id = $1")
            .bind(plan_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        rolled_back.try_get::<Decimal, _>("demand_kg")?,
        Decimal::from(25_000),
        "rollback restores the old name with the same value"
    );
    let new_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_name = 'yield_estimates'
           AND column_name IN ('yield_source', 'sellable_yield_kg', 'price_source', 'average_price_per_kg')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(new_columns, 0, "rollback removes every added column");
    let recorded: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = $1")
            .bind(BRANCHES_MIGRATION)
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        recorded, 0,
        "rollback forgets the migration so it can run again"
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_quick_season_persists_each_answer_without_touching_detail(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let detailed = ten_grade_plan();
    let created = plans::create_quick(&pool, owner.id, 2569, "", &detailed).await?;
    assert_eq!(created.forecast_mode, ForecastMode::Quick);
    assert_eq!(created.quick_estimate, QuickEstimate::default());

    let estimate = QuickEstimate {
        sellable_yield_kg: Some(Decimal::from(20_000)),
        average_price_per_kg: Some(Decimal::from(80)),
        total_cost: Some(Decimal::from(900_000)),
    };
    let saved = plans::save_quick(&pool, owner.id, created.id, &estimate).await?;

    assert_eq!(saved.forecast_mode, ForecastMode::Quick);
    assert_eq!(saved.quick_estimate, estimate);
    assert_eq!(saved.plan, detailed);

    let detailed_mode =
        plans::set_forecast_mode(&pool, owner.id, created.id, ForecastMode::Detailed).await?;
    assert_eq!(detailed_mode.forecast_mode, ForecastMode::Detailed);
    assert_eq!(detailed_mode.quick_estimate, estimate);
    assert_eq!(detailed_mode.plan, detailed);

    let quick_again =
        plans::set_forecast_mode(&pool, owner.id, created.id, ForecastMode::Quick).await?;
    assert_eq!(quick_again.forecast_mode, ForecastMode::Quick);
    assert_eq!(quick_again.quick_estimate, estimate);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_exposed_operation_enforces_owner_scope(pool: PgPool) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let other = users::create(&pool, "other@example.test").await?;
    let plan = ten_grade_plan();
    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;

    assert!(plans::load(&pool, other.id, created.id).await?.is_none());
    assert_not_found(plans::save(&pool, other.id, created.id, &plan).await);
    assert_not_found(
        plans::save_quick(&pool, other.id, created.id, &QuickEstimate::default()).await,
    );
    assert_not_found(
        plans::set_forecast_mode(&pool, other.id, created.id, ForecastMode::Quick).await,
    );
    assert_not_found(
        plans::save_actual_draft(&pool, other.id, created.id, &actual_outcome()).await,
    );
    assert_not_found(
        plans::finalize_with_actual(&pool, other.id, created.id, &actual_outcome()).await,
    );
    assert_not_found(plans::duplicate(&pool, other.id, created.id, 2570, "ขโมยสำเนา", "").await);
    assert_not_found(plans::delete(&pool, other.id, created.id).await);

    assert!(plans::load(&pool, owner.id, created.id).await?.is_some());
    plans::delete(&pool, owner.id, created.id).await?;
    assert!(plans::load(&pool, owner.id, created.id).await?.is_none());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_mutation_of_a_closed_plan_is_refused(pool: PgPool) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let plan = ten_grade_plan();
    let created = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    plans::finalize_with_actual(&pool, owner.id, created.id, &actual_outcome()).await?;

    let closed = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("closed plan remains readable");
    assert!(closed.closed);
    assert_closed(plans::save(&pool, owner.id, created.id, &plan).await);
    assert_closed(plans::save_quick(&pool, owner.id, created.id, &QuickEstimate::default()).await);
    assert_closed(plans::set_forecast_mode(&pool, owner.id, created.id, ForecastMode::Quick).await);
    assert_closed(plans::save_actual_draft(&pool, owner.id, created.id, &actual_outcome()).await);
    let changed_actual = ActualOutcome {
        revenue: Some(Decimal::ONE),
        ..actual_outcome()
    };
    assert_closed(plans::finalize_with_actual(&pool, owner.id, created.id, &changed_actual).await);
    assert_closed(plans::delete(&pool, owner.id, created.id).await);

    let next_season = plans::duplicate(&pool, owner.id, created.id, 2570, "ฤดูกาลถัดไป", "").await?;
    assert!(!next_season.closed);
    assert!(next_season.actual_outcome.is_none());
    assert_eq!(next_season.plan.name, "ฤดูกาลถัดไป");
    assert!(
        plans::load(&pool, owner.id, created.id)
            .await?
            .expect("source still exists")
            .closed
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_is_a_deep_independent_copy(pool: PgPool) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let original_plan = ten_grade_plan();
    let original = plans::create(&pool, owner.id, 2569, "ปีฐาน", &original_plan).await?;
    let duplicate =
        plans::duplicate(&pool, owner.id, original.id, 2570, "สวนปีหน้า", "คัดลอกแล้ว").await?;

    let mut edited = duplicate.plan.clone();
    edited.production.area_rai = Some(Decimal::from(25));
    edited.production.grades[0].name = "เกรดพิเศษ".into();
    edited.variable_costs.push(VariableCostLine {
        name: "รายการใหม่".into(),
        kind: VariableCostKind::Other,
        quantity: Some(Decimal::ONE),
        unit: "ครั้ง".into(),
        unit_price: Some(Decimal::from(999)),
        total_amount: None,
    });
    plans::save(&pool, owner.id, duplicate.id, &edited).await?;

    let reloaded_original = plans::load(&pool, owner.id, original.id)
        .await?
        .expect("original remains");
    let reloaded_duplicate = plans::load(&pool, owner.id, duplicate.id)
        .await?
        .expect("duplicate remains");
    assert_eq!(reloaded_original.plan, original_plan);
    assert_eq!(reloaded_duplicate.plan, edited);
    assert_ne!(reloaded_original.plan, reloaded_duplicate.plan);
    assert_eq!(reloaded_duplicate.season_year, Some(2570));
    assert_eq!(reloaded_duplicate.note, "คัดลอกแล้ว");
    assert_eq!(reloaded_duplicate.forecast_mode, ForecastMode::Quick);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn duplicate_keeps_both_input_sets_but_starts_in_quick_mode(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let detailed = ten_grade_plan();
    let source = plans::create_quick(&pool, owner.id, 2569, "", &detailed).await?;
    let estimate = QuickEstimate {
        sellable_yield_kg: Some(Decimal::from(20_000)),
        average_price_per_kg: Some(Decimal::from(80)),
        total_cost: Some(Decimal::from(900_000)),
    };
    plans::save_quick(&pool, owner.id, source.id, &estimate).await?;
    plans::set_forecast_mode(&pool, owner.id, source.id, ForecastMode::Detailed).await?;

    let duplicate = plans::duplicate(&pool, owner.id, source.id, 2570, "ฤดูถัดไป", "").await?;
    let mut expected_detailed = detailed;
    expected_detailed.name = "ฤดูถัดไป".into();

    assert_eq!(duplicate.forecast_mode, ForecastMode::Quick);
    assert_eq!(duplicate.quick_estimate, estimate);
    assert_eq!(duplicate.plan, expected_detailed);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_returns_only_the_owners_plans_and_closed_state(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let other = users::create(&pool, "other@example.test").await?;
    let first = plans::create(&pool, owner.id, 2568, "ฤดูเก่า", &calc::Plan::default()).await?;
    let named = calc::Plan {
        name: "ฤดูกาลล่าสุด".into(),
        ..calc::Plan::default()
    };
    let latest = plans::create(&pool, owner.id, 2569, "ฤดูล่าสุด", &named).await?;
    plans::finalize_with_actual(&pool, owner.id, first.id, &actual_outcome()).await?;
    plans::create(&pool, other.id, 2569, "", &workbook_sample()).await?;

    let summaries = plans::list(&pool, owner.id).await?;
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].id, latest.id);
    assert_eq!(summaries[0].name, "ฤดูกาลล่าสุด");
    assert_eq!(summaries[0].season_year, Some(2569));
    assert_eq!(summaries[0].note, "ฤดูล่าสุด");
    assert!(!summaries[0].closed);
    assert_eq!(summaries[1].id, first.id);
    assert!(summaries[1].closed);
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn history_is_owner_scoped_chronological_and_keeps_missing_actual_explicit(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let other = users::create(&pool, "other@example.test").await?;

    let legacy = plans::create(&pool, owner.id, 2568, "ปิดก่อนมีผลจริง", &workbook_sample()).await?;
    sqlx::query("UPDATE plans SET closed_at = CURRENT_TIMESTAMP WHERE id = $1 AND owner_id = $2")
        .bind(legacy.id)
        .bind(owner.id)
        .execute(&pool)
        .await?;

    let baseline = plans::create(&pool, owner.id, 2569, "ปีฐาน", &workbook_sample()).await?;
    plans::finalize_with_actual(&pool, owner.id, baseline.id, &actual_outcome()).await?;
    let skipped = plans::create(&pool, owner.id, 2571, "ปีหลังข้ามช่วง", &workbook_sample()).await?;
    let later_actual = ActualOutcome {
        sellable_yield_kg: Some(Decimal::from(19_000)),
        revenue: Some(Decimal::from(1_600_000)),
        total_cost: Some(Decimal::from(1_000_000)),
        note: "ข้อมูลปีหลัง".into(),
    };
    plans::finalize_with_actual(&pool, owner.id, skipped.id, &later_actual).await?;

    let other_plan = plans::create(&pool, other.id, 2570, "ของคนอื่น", &workbook_sample()).await?;
    plans::finalize_with_actual(&pool, other.id, other_plan.id, &actual_outcome()).await?;

    let rows = plans::history(&pool, owner.id).await?;
    assert_eq!(
        rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![legacy.id, baseline.id, skipped.id]
    );
    assert!(rows[0].actual_outcome.is_none());
    assert_eq!(
        rows[1]
            .actual_outcome
            .as_ref()
            .and_then(|actual| actual.forecast.as_ref())
            .and_then(|forecast| forecast.profit),
        calc::analyze(&workbook_sample()).business.net_profit
    );
    assert_eq!(
        rows[2]
            .actual_outcome
            .as_ref()
            .map(|actual| actual.outcome.note.as_str()),
        Some("ข้อมูลปีหลัง")
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn one_owner_cannot_create_two_seasons_for_the_same_year(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let other = users::create(&pool, "other@example.test").await?;
    let plan = calc::Plan {
        name: "สวนรวม".into(),
        ..calc::Plan::default()
    };

    plans::create(&pool, owner.id, 2569, "", &plan).await?;
    assert!(matches!(
        plans::create(&pool, owner.id, 2569, "", &plan).await,
        Err(StoreError::DuplicateSeasonYear)
    ));
    plans::create(&pool, other.id, 2569, "", &plan).await?;
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn metadata_is_editable_only_while_the_season_is_open(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let plan = calc::Plan {
        name: "ชื่อเดิม".into(),
        ..calc::Plan::default()
    };
    let created = plans::create(&pool, owner.id, 2568, "บันทึกเดิม", &plan).await?;

    plans::update_metadata(&pool, owner.id, created.id, 2569, "สวนรวม", "บันทึกใหม่").await?;
    let updated = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("season remains readable");
    assert_eq!(updated.season_year, Some(2569));
    assert_eq!(updated.plan.name, "สวนรวม");
    assert_eq!(updated.note, "บันทึกใหม่");

    plans::finalize_with_actual(&pool, owner.id, created.id, &actual_outcome()).await?;
    assert_closed(
        plans::update_metadata(&pool, owner.id, created.id, 2570, "ห้ามแก้", "ห้ามแก้").await,
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn actual_draft_round_trips_without_closing_the_season(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let created = plans::create_quick(&pool, owner.id, 2569, "", &calc::Plan::default()).await?;
    let draft = ActualOutcome {
        sellable_yield_kg: Some(Decimal::from(18_000)),
        revenue: None,
        total_cost: None,
        note: "กรอกค้างไว้".into(),
    };

    let saved = plans::save_actual_draft(&pool, owner.id, created.id, &draft).await?;
    assert!(!saved.closed);
    let actual = saved.actual_outcome.expect("draft is stored");
    assert_eq!(actual.outcome, draft);
    assert!(!actual.finalized);
    assert!(actual.forecast.is_none());
    assert!(actual.forecast_mode.is_none());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn finalization_is_atomic_snapshots_the_forecast_and_is_idempotent(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let created = plans::create_quick(&pool, owner.id, 2569, "", &calc::Plan::default()).await?;
    let quick = QuickEstimate {
        sellable_yield_kg: Some(Decimal::from(20_000)),
        average_price_per_kg: Some(Decimal::from(80)),
        total_cost: Some(Decimal::from(900_000)),
    };
    plans::save_quick(&pool, owner.id, created.id, &quick).await?;

    let first = plans::finalize_with_actual(&pool, owner.id, created.id, &actual_outcome()).await?;
    let repeated =
        plans::finalize_with_actual(&pool, owner.id, created.id, &actual_outcome()).await?;

    assert!(first.closed);
    assert_eq!(first, repeated);
    let actual = first.actual_outcome.expect("final snapshot exists");
    assert!(actual.finalized);
    assert_eq!(actual.forecast_mode, Some(ForecastMode::Quick));
    let forecast = actual.forecast.expect("forecast is frozen at close");
    assert_eq!(forecast.revenue, Some(Decimal::from(1_600_000)));
    assert_eq!(forecast.profit, Some(Decimal::from(700_000)));
    assert_eq!(forecast.cost_per_kg, Some(Decimal::from(45)));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn closing_on_an_incomplete_forecast_freezes_unknown_as_unknown_and_confirmed_none_as_zero(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let mut plan = calc::Plan {
        production: calc::ProductionPlan {
            yield_source: YieldSource::Direct,
            sellable_yield_kg: Some(Decimal::from(20_000)),
            price_source: PriceSource::Average,
            average_price_per_kg: Some(Decimal::from(80)),
            ..Default::default()
        },
        ..Default::default()
    };

    // Costs never answered: the close succeeds, and the frozen forecast keeps
    // every cost-dependent figure unknown rather than writing a zero.
    let unknown = plans::create(&pool, owner.id, 2569, "", &plan).await?;
    let closed =
        plans::finalize_with_actual(&pool, owner.id, unknown.id, &actual_outcome()).await?;
    assert!(closed.closed);
    let snapshot = closed.actual_outcome.expect("finalized");
    assert!(snapshot.finalized);
    assert_eq!(snapshot.forecast_mode, Some(ForecastMode::Detailed));
    let forecast = snapshot.forecast.expect("forecast is frozen at close");
    assert_eq!(forecast.sellable_yield_kg, Some(Decimal::from(20_000)));
    assert_eq!(forecast.revenue, Some(Decimal::from(1_600_000)));
    assert_eq!(forecast.total_cost, None);
    assert_eq!(forecast.profit, None);
    assert_eq!(forecast.cost_per_kg, None);
    let comparisons = calc::compare(&forecast, &actual_outcome());
    assert_eq!(
        comparisons.iter().filter(|row| row.delta.is_none()).count(),
        3
    );

    // Both sections confirmed empty: a known zero, frozen as such.
    plan.variable_cost_state = CostSectionState::ConfirmedNone;
    plan.fixed_cost_state = CostSectionState::ConfirmedNone;
    let none = plans::create(&pool, owner.id, 2570, "", &plan).await?;
    let closed = plans::finalize_with_actual(&pool, owner.id, none.id, &actual_outcome()).await?;
    let forecast = closed
        .actual_outcome
        .expect("finalized")
        .forecast
        .expect("forecast is frozen at close");
    assert_eq!(forecast.total_cost, Some(Decimal::ZERO));
    assert_eq!(forecast.profit, Some(Decimal::from(1_600_000)));
    assert_eq!(forecast.cost_per_kg, Some(Decimal::ZERO));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn failed_snapshot_write_does_not_close_the_plan(pool: PgPool) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let created = plans::create_quick(&pool, owner.id, 2569, "", &calc::Plan::default()).await?;
    let invalid = ActualOutcome {
        note: "ก".repeat(2_001),
        ..actual_outcome()
    };

    assert!(
        plans::finalize_with_actual(&pool, owner.id, created.id, &invalid)
            .await
            .is_err()
    );
    let reloaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("plan remains");
    assert!(!reloaded.closed);
    assert!(reloaded.actual_outcome.is_none());
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_legacy_closed_season_has_no_invented_actual_outcome(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let created = plans::create(&pool, owner.id, 2568, "", &ten_grade_plan()).await?;
    sqlx::query("UPDATE plans SET closed_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(created.id)
        .execute(&pool)
        .await?;

    let loaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("legacy season remains readable");
    assert!(loaded.closed);
    assert!(loaded.actual_outcome.is_none());
    Ok(())
}
