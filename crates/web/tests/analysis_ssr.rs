#![cfg(feature = "ssr")]

//! Server-rendered proof for the analysis surfaces.
//!
//! These tests read the HTML the server actually sends, so they also stand as
//! the no-script proof: every figure below reaches the reader before any
//! WebAssembly runs.

use calc::{Analysis, KpiKind, Plan, workbook_sample};
use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use rust_decimal::Decimal;
use web::{
    analysis_ui::{PlanAnalysisView, PlanDashboardView},
    plan_form::PlanForm,
    plans::PlanRecord,
};

fn render_with(form: PlanForm, analysis_view: bool) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/dashboard"));
        let record = PlanRecord {
            id: 42,
            season_year: Some(2569),
            note: "ปีทดสอบ".into(),
            closed: false,
            forecast_mode: calc::ForecastMode::Detailed,
            quick_estimate: calc::QuickEstimate::default(),
            form: form.clone(),
        };
        let view = if analysis_view {
            view! { <Router><PlanAnalysisView record/></Router> }.into_any()
        } else {
            view! { <Router><PlanDashboardView record/></Router> }.into_any()
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
    })
}

fn dashboard(form: PlanForm) -> String {
    render_with(form, false)
}

fn analysis(form: PlanForm) -> String {
    render_with(form, true)
}

fn sample_form() -> PlanForm {
    PlanForm::from_plan(&workbook_sample())
}

fn sample_analysis() -> Analysis {
    calc::analyze(&workbook_sample())
}

fn money(value: Decimal) -> String {
    let plain = format!("{value:.2}");
    let (integer, fraction) = plain.split_once('.').unwrap_or((&plain, "00"));
    let (sign, digits) = integer
        .strip_prefix('-')
        .map_or(("", integer), |digits| ("-", digits));
    let mut grouped = String::new();
    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    format!(
        "{sign}{}.{}",
        grouped.chars().rev().collect::<String>(),
        fraction
    )
}

#[test]
fn the_dashboard_renders_every_business_figure_the_workbook_caches() {
    let html = dashboard(sample_form());
    let analysis = sample_analysis();
    let business = analysis.business;

    for (label, value) in [
        ("กำไรสุทธิ", business.net_profit),
        ("ต้นทุนต่อกิโลกรัม", analysis.cost.cost_per_kg),
        ("จุดคุ้มทุน", business.break_even_kg),
        ("รายได้รวม", analysis.revenue.revenue),
        ("ต้นทุนรวม", analysis.cost.total_cost),
        ("กระแสเงินสด", business.operating_cash_flow),
        (
            "ราคาขายเฉลี่ยถ่วงน้ำหนัก",
            analysis.revenue.weighted_price_per_kg,
        ),
        ("ส่วนเกินต่อหน่วย", business.contribution_per_kg),
        ("ส่วนเผื่อความปลอดภัย", business.safety_margin_kg),
        ("ระยะคืนทุน", business.payback_years),
    ] {
        let value = value.expect("the workbook sample computes this figure");
        assert!(
            html.contains(label),
            "{label} is missing from the dashboard"
        );
        assert!(
            html.contains(&money(value)),
            "{label} does not show {}",
            money(value)
        );
    }

    let roi = business.roi.expect("the sample has an investment base");
    assert!(html.contains(&money(roi * Decimal::ONE_HUNDRED)));
    let fulfillment = analysis
        .revenue
        .market_fulfillment
        .expect("the sample states market demand");
    assert!(html.contains(&money(fulfillment * Decimal::ONE_HUNDRED)));
    let health = analysis
        .health
        .overall_score
        .expect("the sample answers all twelve questions");
    assert!(html.contains(&format!("{}/5", money(health))));
}

#[test]
fn missing_investment_and_non_positive_cash_flow_name_the_actual_blocker() {
    let mut no_investment = workbook_sample();
    for line in &mut no_investment.fixed_costs {
        line.investment_base = None;
    }
    let html = dashboard(PlanForm::from_plan(&no_investment));
    assert!(html.contains("กรอกเงินลงทุนอย่างน้อย 1 รายการ"));

    let mut no_cash_return = workbook_sample();
    for grade in &mut no_cash_return.production.grades {
        grade.price_per_kg = Some(Decimal::ZERO);
    }
    let html = dashboard(PlanForm::from_plan(&no_cash_return));
    assert!(html.contains("ยังคืนทุนไม่ได้ เพราะกระแสเงินสดไม่เป็นบวก"));
}

#[test]
fn an_empty_plan_names_what_is_missing_instead_of_showing_a_figure() {
    let html = dashboard(PlanForm::from_plan(&Plan::default()));

    assert!(html.contains("ยังคำนวณไม่ได้"));
    assert!(html.contains("/plans/42/market"));
    assert!(html.contains("/plans/42/health"));
    assert!(!html.contains("hero-value"));
}

#[test]
fn every_kpi_row_reaches_the_reader_with_its_explanation() {
    let html = analysis(sample_form());

    for kind in KpiKind::ALL {
        let label = match kind {
            KpiKind::YieldPerRai => "ผลผลิตต่อไร่",
            KpiKind::YieldPerTree => "ผลผลิตต่อต้น",
            KpiKind::YieldPerLaborDay => "ผลผลิตต่อวันแรงงาน",
            KpiKind::YieldPerFertilizerKg => "ผลผลิตต่อปุ๋ย",
            KpiKind::YieldPerWaterCubicMeter => "ผลผลิตต่อน้ำ",
            KpiKind::YieldPerKwh => "ผลผลิตต่อไฟฟ้า",
            KpiKind::QualityGradeShare => "สัดส่วนเกรดคุณภาพ",
            KpiKind::LossShare => "สัดส่วนสูญเสีย",
            KpiKind::CostPerKg => "ต้นทุนต่อกิโลกรัม",
        };
        assert!(html.contains(label), "{kind:?} is missing");
        assert!(
            html.contains(&format!("อธิบาย{label}")),
            "{kind:?} has no explanation"
        );
    }
    assert_eq!(
        html.matches("คืออะไร").count(),
        html.matches("ไม่ใส่ได้ไหม").count(),
        "every explanation answers all four questions or none"
    );
}

#[test]
fn a_kpi_without_a_target_is_not_graded_and_routes_to_the_targets_screen() {
    let mut plan = workbook_sample();
    plan.targets = calc::KpiTargets::default();
    let html = analysis(PlanForm::from_plan(&plan));

    assert!(html.contains("ยังไม่ได้ตั้งเป้า"));
    assert!(html.contains("/plans/42/targets"));
    assert!(!html.contains("ถึงเป้า"));
    assert!(!html.contains("ต้องปรับปรุง"));
}

#[test]
fn a_kpi_with_a_target_is_graded_against_it() {
    let html = analysis(sample_form());
    let graded = sample_analysis()
        .efficiency
        .iter()
        .filter(|kpi| kpi.verdict.is_some())
        .count();

    assert!(graded > 0, "the sample sets at least one target");
    assert!(html.contains("ถึงเป้า") || html.contains("ต้องปรับปรุง"));
}

#[test]
fn all_six_completeness_rules_render_with_a_route_that_would_fix_them() {
    let html = analysis(sample_form());

    for label in [
        "สัดส่วนเกรดรวมได้ 100%",
        "มีผลผลิตที่ขายได้",
        "ราคาขายสูงกว่าต้นทุนผันแปร",
        "มีเงินลงทุนอย่างน้อย 1 รายการ",
        "ต้นทุนรวมตรงกับรายการที่กรอก",
        "ตอบคำถามสุขภาพครบ 12 ข้อ",
    ] {
        assert!(html.contains(label), "{label} is missing");
    }
    assert!(html.contains("/plans/42/production"));
    assert!(html.contains("/plans/42/variable-costs"));
    assert!(html.contains("/plans/42/fixed-costs"));

    let readiness = match sample_analysis().checks.overall {
        calc::Readiness::Ready => "พร้อมใช้ตัดสินใจ",
        calc::Readiness::NeedsReview => "ยังต้องตรวจ",
    };
    assert!(
        html.contains(readiness),
        "the rendered readiness does not match the engine"
    );
}

#[test]
fn both_tax_methods_render_with_the_cheaper_one_marked_and_the_disclaimer_visible() {
    let html = analysis(sample_form());
    let tax = sample_analysis().tax;

    assert!(html.contains("หักค่าใช้จ่ายตามจริง"));
    assert!(html.contains("หักค่าใช้จ่ายแบบเหมา"));
    assert!(html.contains("ไม่ใช่คำแนะนำทางภาษี"));

    let actual = tax.actual_expense.estimated_tax.expect("actual method");
    let flat = tax.flat_sixty_percent.estimated_tax.expect("flat method");
    assert!(html.contains(&money(actual)));
    assert!(html.contains(&money(flat)));
    // The phrase also appears inside the tax explanation, so the marker to
    // assert on is the class the cheaper card actually carries.
    assert_eq!(
        html.contains("tax-method cheaper"),
        actual != flat,
        "the cheaper method is marked only when one is actually cheaper"
    );
}

#[test]
fn all_twenty_five_scenario_cells_are_reachable_in_the_full_table() {
    let html = analysis(sample_form());
    let scenario = sample_analysis().scenario;

    assert!(html.contains("ดูตารางทั้งหมด"));
    for row in scenario.profits {
        for cell in row {
            let cell = cell.expect("the workbook sample fills every cell");
            assert!(
                html.contains(&money(cell)),
                "the matrix is missing {}",
                money(cell)
            );
        }
    }
}

#[test]
fn the_slider_starting_position_is_the_unchanged_centre_of_the_matrix() {
    let html = analysis(sample_form());
    let scenario = sample_analysis().scenario;
    let centre = scenario.profits[2][2].expect("no change is a computed cell");

    assert_eq!(scenario.yield_changes[2], Decimal::ZERO);
    assert_eq!(scenario.price_changes[2], Decimal::ZERO);
    assert!(
        html.contains(&money(centre)),
        "the slider does not agree with the matrix it reads from"
    );
}

#[test]
fn an_unanalysable_plan_shows_no_tab_content_it_cannot_support() {
    let html = analysis(PlanForm::from_plan(&Plan::default()));

    assert!(html.contains("ยังคำนวณไม่ได้") || html.contains("ยังไม่มีข้อมูล"));
    assert!(
        !html.contains("tax-method cheaper"),
        "neither method can be cheaper when neither has a figure"
    );
    assert!(
        !html.contains("ถึงเป้า"),
        "no KPI is graded when nothing can be calculated"
    );
}
