use calc::{ActualOutcome, AssetFacts, AssetKind, ForecastMode, workbook_sample};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use store::{
    assets,
    plans::{self, StoreError},
    users,
};

fn equipment(name: &str) -> AssetFacts {
    AssetFacts {
        name: name.into(),
        kind: AssetKind::Equipment,
        original_cost: Decimal::from(100_000),
        start_year: 2568,
        useful_life_years: Some(5),
        residual_value: None,
        retired_year: None,
    }
}

fn actual() -> ActualOutcome {
    ActualOutcome {
        sellable_yield_kg: Some(Decimal::from(18_000)),
        revenue: Some(Decimal::from(1_500_000)),
        total_cost: Some(Decimal::from(900_000)),
        note: String::new(),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn owner_assets_are_reusable_explicitly_selected_and_owner_scoped(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let other = users::create(&pool, "other@example.test").await?;
    let asset = assets::create(&pool, owner.id, &equipment("ระบบน้ำ")).await?;
    let first = plans::create(&pool, owner.id, 2569, "", &workbook_sample()).await?;
    let second = plans::create(&pool, owner.id, 2570, "", &workbook_sample()).await?;

    assert!(assets::load(&pool, other.id, asset.id).await?.is_none());
    assert!(matches!(
        assets::update(&pool, other.id, asset.id, &equipment("แก้ไม่ได้")).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        assets::set_selected(&pool, other.id, first.id, asset.id, true).await,
        Err(StoreError::NotFound)
    ));

    for plan in [first.id, second.id] {
        let choices = assets::choices_for_plan(&pool, owner.id, plan).await?;
        assert_eq!(choices.len(), 1);
        assert!(
            !choices[0].selected,
            "assets default to excluded per season"
        );
        assets::set_selected(&pool, owner.id, plan, asset.id, true).await?;
        assert!(assets::choices_for_plan(&pool, owner.id, plan).await?[0].selected);
    }

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn selected_asset_and_starting_capital_have_separate_calculation_roles(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let asset = assets::create(&pool, owner.id, &equipment("รถตัดหญ้า")).await?;
    let created = plans::create(&pool, owner.id, 2569, "", &workbook_sample()).await?;
    assets::set_selected(&pool, owner.id, created.id, asset.id, true).await?;
    let stored =
        plans::save_starting_capital(&pool, owner.id, created.id, Some(Decimal::from(50_000)))
            .await?;

    let analysis = calc::analyze_with_assets(
        &stored.plan,
        &stored.asset_allocations,
        stored.starting_capital,
    );
    assert_eq!(
        analysis.cost.manual_fixed_cost,
        Some(Decimal::from(255_100))
    );
    assert_eq!(
        analysis.cost.asset_depreciation,
        Some(Decimal::from(20_000))
    );
    assert_eq!(analysis.cost.fixed_cost, Some(Decimal::from(275_100)));
    assert_eq!(
        analysis.cost.manual_investment_base,
        Some(Decimal::from(700_000))
    );
    assert_eq!(
        analysis.cost.asset_investment_base,
        Some(Decimal::from(100_000))
    );
    assert_eq!(analysis.cost.starting_capital, Some(Decimal::from(50_000)));
    assert_eq!(analysis.cost.investment_base, Some(Decimal::from(850_000)));

    let quick = calc::forecast_metrics_with_assets(
        ForecastMode::Quick,
        &calc::QuickEstimate {
            sellable_yield_kg: Some(Decimal::from(10)),
            average_price_per_kg: Some(Decimal::from(20)),
            total_cost: Some(Decimal::from(30)),
        },
        &stored.plan,
        &stored.asset_allocations,
        stored.starting_capital,
    );
    assert_eq!(quick.total_cost, Some(Decimal::from(30)));
    assert_eq!(quick.profit, Some(Decimal::from(170)));
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn closing_snapshots_asset_facts_and_rejects_later_season_edits(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let asset = assets::create(&pool, owner.id, &equipment("ปั๊มน้ำ")).await?;
    let created = plans::create(&pool, owner.id, 2569, "", &workbook_sample()).await?;
    assets::set_selected(&pool, owner.id, created.id, asset.id, true).await?;
    plans::save_starting_capital(&pool, owner.id, created.id, Some(Decimal::from(50_000))).await?;

    let closed = plans::finalize_with_actual(&pool, owner.id, created.id, &actual()).await?;
    assert!(closed.closed);
    assert_eq!(
        closed.asset_allocations[0].facts.original_cost,
        Decimal::from(100_000)
    );
    assert_eq!(
        closed.asset_allocations[0].annual_depreciation,
        Decimal::from(20_000)
    );

    let mut changed = equipment("ปั๊มน้ำรุ่นใหม่");
    changed.original_cost = Decimal::from(200_000);
    changed.retired_year = Some(2570);
    assets::update(&pool, owner.id, asset.id, &changed).await?;
    let reloaded = plans::load(&pool, owner.id, created.id)
        .await?
        .expect("closed season remains readable");
    assert_eq!(reloaded.asset_allocations, closed.asset_allocations);
    assert!(matches!(
        assets::set_selected(&pool, owner.id, created.id, asset.id, false).await,
        Err(StoreError::Closed)
    ));
    assert!(matches!(
        plans::save_starting_capital(&pool, owner.id, created.id, None).await,
        Err(StoreError::Closed)
    ));

    let duplicate = plans::duplicate(&pool, owner.id, created.id, 2570, "ฤดูใหม่", "").await?;
    assert_eq!(duplicate.starting_capital, Some(Decimal::from(50_000)));
    assert!(duplicate.asset_allocations.is_empty());
    assert!(
        assets::choices_for_plan(&pool, owner.id, duplicate.id)
            .await?
            .iter()
            .all(|choice| !choice.selected)
    );

    let snapshot = sqlx::query(
        "SELECT source_asset_id, name, original_cost FROM season_asset_snapshots WHERE plan_id = $1",
    )
    .bind(created.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(snapshot.try_get::<i64, _>("source_asset_id")?, asset.id);
    assert_eq!(snapshot.try_get::<String, _>("name")?, "ปั๊มน้ำ");
    assert_eq!(
        snapshot.try_get::<Decimal, _>("original_cost")?,
        Decimal::from(100_000)
    );
    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn owned_land_adds_investment_context_without_depreciation(
    pool: PgPool,
) -> Result<(), StoreError> {
    let owner = users::create(&pool, "owner@example.test").await?;
    let land = AssetFacts {
        name: "ที่ดินสวน".into(),
        kind: AssetKind::OwnedLand,
        original_cost: Decimal::from(2_000_000),
        start_year: 2550,
        useful_life_years: None,
        residual_value: None,
        retired_year: None,
    };
    let asset = assets::create(&pool, owner.id, &land).await?;
    let created = plans::create(&pool, owner.id, 2569, "", &workbook_sample()).await?;
    assets::set_selected(&pool, owner.id, created.id, asset.id, true).await?;
    let stored = plans::load(&pool, owner.id, created.id).await?.unwrap();
    let analysis = calc::analyze_with_assets(&stored.plan, &stored.asset_allocations, None);

    assert_eq!(analysis.cost.asset_depreciation, Some(Decimal::ZERO));
    assert_eq!(
        analysis.cost.asset_investment_base,
        Some(Decimal::from(2_000_000))
    );
    assert_eq!(analysis.cost.fixed_cost, analysis.cost.manual_fixed_cost);
    Ok(())
}
