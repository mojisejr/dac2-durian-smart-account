#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::{components::Router, location::RequestUrl};
use rust_decimal::Decimal;
use web::{
    analysis_ui::PlanDashboardView,
    assets::{AssetPageData, AssetPageView, AssetRow},
    plan_form::PlanForm,
    plans::PlanRecord,
};

fn allocation(kind: calc::AssetKind) -> calc::AssetAllocation {
    let land = kind == calc::AssetKind::OwnedLand;
    calc::AssetAllocation {
        facts: calc::AssetFacts {
            name: if land {
                "ที่ดินสวน"
            } else {
                "ระบบน้ำ"
            }
            .into(),
            kind,
            original_cost: Decimal::from(if land { 2_000_000 } else { 100_000 }),
            start_year: 2568,
            useful_life_years: (!land).then_some(5),
            residual_value: None,
            retired_year: None,
        },
        season_year: 2569,
        annual_depreciation: Decimal::from(if land { 0 } else { 20_000 }),
        investment_value: Decimal::from(if land { 2_000_000 } else { 100_000 }),
        residual_assumed_zero: !land,
    }
}

fn render_assets(data: AssetPageData) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/assets"));
        let view = view! { <Router><AssetPageView data/></Router> };
        let mut html = String::new();
        view.to_html_with_buf(
            &mut html,
            &mut Position::FirstChild,
            true,
            false,
            Vec::new(),
        );
        html
    })
}

fn data(selected: bool, closed: bool) -> AssetPageData {
    AssetPageData {
        plan_id: 42,
        plan_name: "สวนทดสอบ".into(),
        season_year: Some(2569),
        closed,
        forecast_mode: calc::ForecastMode::Detailed,
        starting_capital: Some(Decimal::from(50_000)),
        manual_fixed_cost: Some(Decimal::from(255_100)),
        manual_investment_base: Some(Decimal::from(700_000)),
        assets: vec![AssetRow {
            id: 7,
            facts: allocation(calc::AssetKind::Equipment).facts,
            selected,
            allocation: Some(allocation(calc::AssetKind::Equipment)),
        }],
    }
}

#[test]
fn open_asset_page_names_every_role_and_requires_explicit_season_inclusion() {
    let html = render_assets(data(false, false));
    for text in [
        "ต้นทุนคงที่ที่กรอกเอง",
        "ค่าเสื่อมจากสินทรัพย์ที่เลือก",
        "เงินลงทุนที่กรอกในรายการเดิม",
        "มูลค่าสินทรัพย์ที่เลือก",
        "เงินทุนเริ่มต้น",
        "ยังไม่รวม",
        "รวมในฤดูนี้",
        "ระบบจะไม่เดาหรือลบรายการเดิมให้",
        "ไม่ใช่ค่าเสื่อมทางภาษีหรือราคาตลาด",
    ] {
        assert!(html.contains(text), "missing {text}");
    }
}

#[test]
fn closed_asset_page_is_a_snapshot_without_edit_controls() {
    let html = render_assets(data(true, true));
    assert!(html.contains("ข้อมูลนี้ถูกเก็บพร้อมตอนปิดฤดู"));
    assert!(html.contains("รวมในฤดูนี้"));
    assert!(!html.contains("เอาออกจากฤดูนี้"));
    assert!(!html.contains("แก้ข้อมูลหรือระบุปีเลิกใช้"));
    assert!(!html.contains("บันทึกเงินทุนเริ่มต้น"));
}

#[test]
fn selected_assets_reach_the_detailed_dashboard_calculation() {
    let asset = allocation(calc::AssetKind::Equipment);
    let html = Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42"));
        let view = view! {
            <Router><PlanDashboardView record=PlanRecord {
                id: 42,
                season_year: Some(2569),
                note: String::new(),
                closed: false,
                forecast_mode: calc::ForecastMode::Detailed,
                quick_estimate: calc::QuickEstimate::default(),
                starting_capital: Some(Decimal::from(50_000)),
                asset_allocations: vec![asset],
                actual_outcome: None,
                form: PlanForm::from_plan(&calc::workbook_sample()),
            }/></Router>
        };
        let mut html = String::new();
        view.to_html_with_buf(
            &mut html,
            &mut Position::FirstChild,
            true,
            false,
            Vec::new(),
        );
        html
    });
    assert!(html.contains("831,275.00 บาท"));
}
