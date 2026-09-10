use calc::{Grade, VariableCostKind, VariableCostLine, workbook_sample};
use rust_decimal::Decimal;
use sqlx::PgPool;
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
    assert!(loaded.plan.market.demand_kg.is_none());
    assert!(loaded.plan.production.grades.is_empty());
    assert!(loaded.plan.targets.yield_per_rai.is_none());
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
    assert_not_found(plans::close(&pool, other.id, created.id).await);
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
    plans::close(&pool, owner.id, created.id).await?;

    let closed = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("closed plan remains readable");
    assert!(closed.closed);
    assert_closed(plans::save(&pool, owner.id, created.id, &plan).await);
    assert_closed(plans::close(&pool, owner.id, created.id).await);
    assert_closed(plans::delete(&pool, owner.id, created.id).await);

    let next_season = plans::duplicate(&pool, owner.id, created.id, 2570, "ฤดูกาลถัดไป", "").await?;
    assert!(!next_season.closed);
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
    plans::close(&pool, owner.id, first.id).await?;
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

    plans::close(&pool, owner.id, created.id).await?;
    assert_closed(
        plans::update_metadata(&pool, owner.id, created.id, 2570, "ห้ามแก้", "ห้ามแก้").await,
    );
    Ok(())
}
