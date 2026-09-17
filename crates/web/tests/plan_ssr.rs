#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use web::{
    plan_form::{GradeEntry, PlanForm},
    plan_ui::{
        ActualCloseView, ActualComparisonView, ActualReviewView, DemoPage,
        DetailedModeActiveNotice, PlanField, PlanHub, PlanSectionView, QuickQuestionView,
        QuickResultView, SeasonHistoryView,
    },
    plans::{ActualOutcomeRecord, PlanRecord, SeasonHistoryItem},
};

const SECTIONS: [(&str, &str); 7] = [
    ("market", "ตลาด"),
    ("production", "ผลผลิตและราคา"),
    ("expenses", "ค่าใช้จ่ายที่จำได้"),
    ("variable-costs", "ต้นทุนผันแปร"),
    ("fixed-costs", "ต้นทุนคงที่"),
    ("targets", "เป้าหมาย"),
    ("health", "สุขภาพสวน"),
];

fn render(section: &str, closed: bool, form: PlanForm) -> String {
    render_with_assets(section, closed, form, Vec::new())
}

fn included_asset() -> calc::AssetAllocation {
    calc::allocate(
        &calc::AssetFacts {
            name: "ระบบน้ำ".into(),
            kind: calc::AssetKind::Equipment,
            original_cost: 100_000.into(),
            start_year: 2568,
            useful_life_years: Some(5),
            residual_value: None,
            retired_year: None,
        },
        2569,
    )
    .unwrap()
    .unwrap()
}

fn render_with_assets(
    section: &str,
    closed: bool,
    form: PlanForm,
    asset_allocations: Vec<calc::AssetAllocation>,
) -> String {
    let section = section.to_owned();
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/test"));
        let view = view! {
            <Router>
                <PlanSectionView
                    record=PlanRecord {
                        id: 42,
                        season_year: Some(2569),
                        note: "ปีทดสอบ".into(),
                        closed,
                        forecast_mode: calc::ForecastMode::Detailed,
                        quick_estimate: calc::QuickEstimate::default(),
                        starting_capital: None,
                        asset_allocations: asset_allocations.clone(),
                        actual_outcome: None,
                        form: form.clone(),
                    }
                    section=section.clone()
                />
            </Router>
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

fn render_demo() -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/demo"));
        let view = view! { <Router><DemoPage/></Router> };
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

fn render_invalid_guided_field() -> String {
    Owner::new().with(move || {
        let view = view! {
            <PlanField
                field_id="proof-field"
                label="จ่ายรวมเท่าไร"
                formal_term="ต้นทุนรวม"
                hint="กรอกยอดที่จ่ายทั้งฤดู"
                example="ตัวอย่าง 900000"
                outcome="ใช้คำนวณกำไร"
                unit="บาท"
                numeric=true
                value=Signal::derive(|| "ไม่ใช่ตัวเลข".to_owned())
                on_value=Callback::new(|_| {})
                closed=false
            />
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

fn quick_record(estimate: calc::QuickEstimate, closed: bool) -> PlanRecord {
    PlanRecord {
        id: 42,
        season_year: Some(2569),
        note: "ปีทดสอบ".into(),
        closed,
        forecast_mode: calc::ForecastMode::Quick,
        quick_estimate: estimate,
        starting_capital: None,
        asset_allocations: Vec::new(),
        actual_outcome: None,
        form: PlanForm::from_plan(&calc::Plan::default()),
    }
}

fn render_quick_question(step: &str, estimate: calc::QuickEstimate) -> String {
    let step = step.to_owned();
    Owner::new().with(move || {
        let url = format!("/plans/42/quick/{step}");
        provide_context(RequestUrl::new(&url));
        let view = view! {
            <Router>
                <QuickQuestionView record=quick_record(estimate, false) step=step.clone()/>
            </Router>
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

fn render_quick_result(estimate: calc::QuickEstimate) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/quick/result"));
        let view = view! {
            <Router><QuickResultView record=quick_record(estimate, false)/></Router>
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

fn render_hub(form: PlanForm, closed: bool) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42"));
        let view = view! {
            <Router>
                <PlanHub record=PlanRecord {
                    id: 42,
                    season_year: Some(2569),
                    note: String::new(),
                    closed,
                    forecast_mode: calc::ForecastMode::Detailed,
                    quick_estimate: calc::QuickEstimate::default(),
                    starting_capital: None,
                    asset_allocations: Vec::new(),
                    actual_outcome: None,
                    form: form.clone(),
                }/>
            </Router>
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

fn render_detailed_mode_notice() -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/quick/production"));
        let mut record = quick_record(calc::QuickEstimate::default(), false);
        record.forecast_mode = calc::ForecastMode::Detailed;
        let view = view! { <Router><DetailedModeActiveNotice record/></Router> };
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

fn actual_record(finalized: bool) -> PlanRecord {
    let quick = calc::QuickEstimate {
        sellable_yield_kg: Some(rust_decimal::Decimal::from(20_000)),
        average_price_per_kg: Some(rust_decimal::Decimal::from(80)),
        total_cost: Some(rust_decimal::Decimal::from(900_000)),
    };
    let outcome = calc::ActualOutcome {
        sellable_yield_kg: Some(rust_decimal::Decimal::from(18_000)),
        revenue: Some(rust_decimal::Decimal::from(1_530_000)),
        total_cost: Some(rust_decimal::Decimal::from(990_000)),
        note: "ผลผลิตน้อยกว่าคาด".into(),
    };
    let forecast = finalized
        .then(|| calc::forecast_metrics(calc::ForecastMode::Quick, &quick, &calc::Plan::default()));
    PlanRecord {
        id: 42,
        season_year: Some(2569),
        note: String::new(),
        closed: finalized,
        forecast_mode: calc::ForecastMode::Quick,
        quick_estimate: quick,
        starting_capital: None,
        asset_allocations: Vec::new(),
        actual_outcome: Some(ActualOutcomeRecord {
            outcome,
            finalized,
            forecast_mode: finalized.then_some(calc::ForecastMode::Quick),
            forecast,
        }),
        form: PlanForm::from_plan(&calc::Plan::default()),
    }
}

fn render_actual(view: &str, record: PlanRecord) -> String {
    let view = view.to_owned();
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/close"));
        let rendered = match view.as_str() {
            "entry" => view! { <Router><ActualCloseView record=record/></Router> }.into_any(),
            "review" => view! { <Router><ActualReviewView record=record/></Router> }.into_any(),
            _ => view! { <Router><ActualComparisonView record=record/></Router> }.into_any(),
        };
        let mut html = String::new();
        rendered.to_html_with_buf(
            &mut html,
            &mut Position::FirstChild,
            true,
            false,
            Vec::new(),
        );
        html
    })
}

fn outcome_metrics(yield_kg: i64, revenue: i64, cost: i64) -> calc::OutcomeMetrics {
    let yield_kg = rust_decimal::Decimal::from(yield_kg);
    let revenue = rust_decimal::Decimal::from(revenue);
    let cost = rust_decimal::Decimal::from(cost);
    calc::OutcomeMetrics {
        sellable_yield_kg: Some(yield_kg),
        revenue: Some(revenue),
        total_cost: Some(cost),
        profit: Some(revenue - cost),
        average_price_per_kg: (!yield_kg.is_zero()).then(|| revenue / yield_kg),
        cost_per_kg: (!yield_kg.is_zero()).then(|| cost / yield_kg),
    }
}

fn history_item(
    id: i64,
    year: i32,
    name: &str,
    actual: Option<calc::OutcomeMetrics>,
    forecast: Option<calc::OutcomeMetrics>,
) -> SeasonHistoryItem {
    SeasonHistoryItem {
        id,
        name: name.into(),
        season_year: Some(year),
        actual_outcome: actual.map(|metrics| ActualOutcomeRecord {
            outcome: calc::ActualOutcome {
                sellable_yield_kg: metrics.sellable_yield_kg,
                revenue: metrics.revenue,
                total_cost: metrics.total_cost,
                note: format!("บันทึกฤดู {year}"),
            },
            finalized: true,
            forecast_mode: Some(calc::ForecastMode::Quick),
            forecast,
        }),
    }
}

fn render_history(items: Vec<SeasonHistoryItem>) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/history"));
        let view =
            view! { <Router><SeasonHistoryView items=items.clone() return_to=None/></Router> };
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

fn render_history_from(items: Vec<SeasonHistoryItem>, return_to: i64) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new(&format!("/history?from={return_to}")));
        let view = view! { <Router><SeasonHistoryView items=items.clone() return_to=Some(return_to)/></Router> };
        let mut html = String::new();
        view.to_html_with_buf(&mut html, &mut Position::FirstChild, true, false, Vec::new());
        html
    })
}

#[test]
fn demonstration_is_an_editable_browser_surface_not_a_persisting_form() {
    let html = render_demo();
    assert!(html.contains("ลองก่อน โดยไม่บันทึก"));
    assert!(html.contains("คืนค่าตัวอย่าง"));
    assert!(html.contains("สร้างฤดูกาลของฉัน"));
    assert!(!html.contains("<form"));
    assert!(html.contains("ผลผลิตต่อต้น"));
    assert!(html.contains("ค่านี้เปลี่ยนผลผลิต รายได้ และกำไร"));
    assert!(html.contains("aria-describedby=\"demo-fruits-help\""));
    assert!(html.contains("id=\"demo-fruits-help\""));
}

#[test]
fn guided_field_associates_persistent_help_and_error_with_the_input() {
    let html = render_invalid_guided_field();
    assert!(html.contains("จ่ายรวมเท่าไร"));
    assert!(html.contains("ต้นทุนรวม"));
    assert!(html.contains("กรอกยอดที่จ่ายทั้งฤดู"));
    assert!(html.contains("ตัวอย่าง 900000"));
    assert!(html.contains("ใช้คำนวณกำไร"));
    assert!(html.contains("aria-invalid=\"true\""));
    assert!(html.contains("aria-describedby=\"proof-field-help proof-field-error\""));
    assert!(html.contains("id=\"proof-field-help\""));
    assert!(html.contains("id=\"proof-field-error\""));
}

#[test]
fn actual_entry_prefills_a_saved_draft_but_does_not_close_directly() {
    let html = render_actual("entry", actual_record(false));
    assert!(html.contains("บันทึกผลจริง"));
    assert!(html.contains("18,000") || html.contains("18000"));
    assert!(html.contains("บันทึกและตรวจทาน"));
    assert!(html.contains("ยกเลิก · เก็บฤดูกาลไว้เปิดอยู่"));
    assert!(!html.contains("ยืนยันผลจริงและปิดฤดูกาล"));
}

#[test]
fn actual_review_names_the_irreversible_step_and_all_source_facts() {
    let html = render_actual("review", actual_record(false));
    assert!(html.contains("ตรวจทานก่อนปิดฤดูกาล"));
    assert!(html.contains("18,000.00 กก."));
    assert!(html.contains("1,530,000 บาท"));
    assert!(html.contains("990,000 บาท"));
    assert!(html.contains("540,000 บาท"));
    assert!(html.contains("ยืนยันผลจริงและปิดฤดูกาล"));
    assert!(html.contains("กลับไปแก้ผลจริง"));
}

#[test]
fn final_comparison_names_source_formula_direction_and_unavailable_rules() {
    let html = render_actual("comparison", actual_record(true));
    assert!(html.contains("ผลจริงเทียบประมาณการ"));
    assert!(html.contains("แหล่งประมาณการ: ประมาณการเร็ว"));
    assert!(html.contains("ผลต่าง = ผลจริง - ประมาณการ"));
    assert!(html.contains("ต่ำกว่าประมาณการ 2,000.00 กก."));
    assert!(html.contains("สูงกว่าประมาณการ 90,000 บาท"));
    assert!(html.contains("ต่ำกว่าประมาณการ 160,000 บาท"));
    assert!(html.contains("ผลผลิตน้อยกว่าคาด"));
}

#[test]
fn legacy_closed_season_never_renders_synthetic_zero_actuals() {
    let html = render_actual(
        "comparison",
        quick_record(calc::QuickEstimate::default(), true),
    );
    assert!(html.contains("ไม่มีผลจริงที่บันทึกไว้"));
    assert!(html.contains("ไม่เติมศูนย์หรือสร้างตัวเลขแทน"));
    assert!(!html.contains("0 บาท"));
}

#[test]
fn season_history_labels_legacy_baseline_skipped_year_and_fixed_cues() {
    let baseline = outcome_metrics(20_000, 1_600_000, 900_000);
    let later = outcome_metrics(18_000, 1_530_000, 990_000);
    let html = render_history(vec![
        history_item(
            30,
            2571,
            "ปีหลัง",
            Some(later.clone()),
            Some(baseline.clone()),
        ),
        history_item(10, 2568, "ปิดแบบเดิม", None, None),
        history_item(20, 2569, "ปีฐาน", Some(baseline.clone()), Some(baseline)),
    ]);

    assert!(html.contains("ประวัติฤดูกาล"));
    assert!(html.contains("ไม่มีผลจริง"));
    assert!(html.contains("ไม่เติมศูนย์หรือสร้างแนวโน้มแทน"));
    assert!(html.contains("ปีฐานและยังไม่สรุปว่าเป็นแนวโน้ม"));
    assert!(html.contains("มีปีที่ข้ามระหว่างสองผลจริง"));
    assert!(html.contains("เงินที่จ่ายไปทั้งหมด สูงกว่าฤดูกาลก่อน 10.00%"));
    assert!(html.contains("กิโลที่ขายได้ ต่ำกว่าฤดูกาลก่อน 10.00%"));
    // The formal term sits under the plain words in the table's row headings.
    assert!(html.contains("กิโลที่ขายได้<small class=\"formal-term\">ผลผลิตที่ขายได้</small>"));
    assert!(html.contains("สูตร: ((ผลจริงปีนี้ - ผลจริงปีก่อน) ÷ |ผลจริงปีก่อน|) × 100"));
    assert!(html.contains("<table"));
    assert!(html.contains("scope=\"col\""));
    assert!(html.contains("scope=\"row\""));

    let legacy_position = html.find("ปิดแบบเดิม").expect("legacy row");
    let baseline_position = html.find("บันทึกฤดู 2569").expect("baseline row");
    let later_position = html.find("บันทึกฤดู 2571").expect("later row");
    assert!(legacy_position < baseline_position && baseline_position < later_position);
}

#[test]
fn season_history_names_zero_denominator_as_unavailable_instead_of_zero_percent() {
    let zero = outcome_metrics(0, 0, 0);
    let later = outcome_metrics(10, 100, 50);
    let html = render_history(vec![
        history_item(1, 2569, "ปีศูนย์", Some(zero.clone()), Some(zero)),
        history_item(2, 2570, "ปีถัดไป", Some(later.clone()), Some(later)),
    ]);

    assert!(html.contains("คิดเปอร์เซ็นต์ไม่ได้ เพราะค่าฤดูกาลก่อนเป็นศูนย์"));
    assert!(html.contains("ไม่มี % เพราะฐานเป็น 0"));
    assert!(!html.contains("สูงกว่าฤดูกาลก่อน 0.00%"));
}

#[test]
fn history_opened_from_a_season_has_an_explicit_route_back() {
    let metrics = outcome_metrics(10, 100, 50);
    let html = render_history_from(
        vec![history_item(
            42,
            2569,
            "ปีฐาน",
            Some(metrics.clone()),
            Some(metrics),
        )],
        42,
    );

    assert!(html.contains("‹ กลับไปฤดูกาลที่เปิดอยู่"));
    assert!(html.contains("href=\"/plans/42\""));
}

#[test]
fn detailed_hub_moves_targets_under_an_explicit_advanced_area() {
    let html = render_hub(PlanForm::from_plan(&calc::workbook_sample()), false);

    assert!(html.contains("การวางแผนขั้นสูง (ไม่บังคับ)"));
    assert!(html.contains("เป้าหมาย KPI"));
    assert!(html.contains("ค่าที่ตั้งไว้เดิมยังอยู่และแก้ได้ที่นี่"));
    assert!(html.contains("/plans/42/targets"));
    assert_eq!(html.matches("ตัวเลขเปรียบเทียบที่เจ้าของกำหนด").count(), 0);
    assert!(html.contains("ขั้นที่แนะนำ"));
    assert!(html.contains("ดูและแก้ข้อมูลทั้งหมด"));
    assert!(html.contains("พอคำนวณรายได้แล้ว"));
    assert!(html.contains("พอคำนวณค่าใช้จ่ายตามการผลิตแล้ว"));
    assert!(html.contains("พอคำนวณค่าใช้จ่ายประจำแล้ว"));
    assert!(!html.contains(">ครบ<"));
    assert!(!html.contains("ยังไม่ครบ"));
}

#[test]
fn closed_hub_keeps_truthful_readiness_without_edit_or_mode_actions() {
    let html = render_hub(PlanForm::from_plan(&calc::workbook_sample()), true);
    assert!(html.contains("ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"));
    assert!(html.contains("พอคำนวณรายได้แล้ว"));
    assert!(!html.contains("เปลี่ยนเป็นประมาณการเร็ว"));
    assert!(!html.contains("ยังไม่ครบ"));
}

#[test]
fn all_seven_input_routes_render_an_empty_editable_state() {
    for (section, title) in SECTIONS {
        let html = render(section, false, PlanForm::from_plan(&calc::Plan::default()));
        assert!(html.contains(title), "{section}");
        assert!(html.contains("บันทึกส่วนนี้"), "{section}");
        assert!(
            html.contains("<input") || html.contains("<button"),
            "{section}"
        );
        assert!(!html.contains("ฤดูกาลนี้ปิดแล้ว"), "{section}");
        assert!(html.contains("aria-current=\"page\""), "{section}");
        assert!(html.contains("ยังคำนวณกำไรสุทธิไม่ได้"), "{section}");
        assert!(!html.contains("ยอดรวมสด"), "{section}");
    }
}

#[test]
fn all_seven_input_routes_render_closed_values_without_edit_controls() {
    for (section, title) in SECTIONS {
        let html = render(section, true, PlanForm::from_plan(&calc::workbook_sample()));
        assert!(html.contains(title), "{section}");
        assert!(html.contains("ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"), "{section}");
        assert!(!html.contains("บันทึกส่วนนี้"), "{section}");
        assert!(html.contains("readonly-value"), "{section}: {html}");
    }
}

// A closed season's numbers read like the open season's did after a blur:
// thousands separated, never the raw stored string. dac2-ui-polish-001.
#[test]
fn a_closed_season_separates_thousands_in_its_read_only_values() {
    let mut form = PlanForm::from_plan(&calc::workbook_sample());
    form.production.yield_source = calc::YieldSource::Direct;
    form.production.sellable_yield_kg = "18000".into();
    let html = render("production", true, form);
    assert!(
        html.contains("<p class=\"readonly-value\">18,000 กก.</p>"),
        "{html}"
    );
    assert!(!html.contains(">18000 กก.<"), "{html}");
}

// The season's year, name and note are one row under the heading with the
// form behind a disclosure, not a card of three fields at the top of the
// page; a closed season shows the row with nothing to open.
#[test]
fn the_hub_keeps_the_season_details_in_one_row() {
    let open = render_hub(PlanForm::from_plan(&calc::workbook_sample()), false);
    assert!(!open.contains("รายละเอียดฤดูกาล"), "{open}");
    let row = open.find("class=\"season-row\"").expect("season row");
    let mode = open.find("โหมดที่ใช้อยู่").expect("mode card");
    assert!(row < mode, "the row precedes the mode card");
    let summary = &open[row..mode];
    assert!(
        summary.contains("ตัวอย่างจากแบบคำนวณ · ฤดูกาล 2569"),
        "{summary}"
    );
    assert!(summary.contains(">แก้<"), "{summary}");
    assert!(
        summary.contains("name=\"season_year\""),
        "the form is still there, behind the disclosure"
    );
    assert!(summary.contains("บันทึกรายละเอียด"));

    let closed = render_hub(PlanForm::from_plan(&calc::workbook_sample()), true);
    assert!(closed.contains("class=\"season-row-static\""), "{closed}");
    assert!(
        !closed.contains("season_year"),
        "no form on a closed season"
    );
}

#[test]
fn fixed_costs_explain_that_investment_is_optional_per_item() {
    let html = render(
        "fixed-costs",
        false,
        PlanForm::from_plan(&calc::workbook_sample()),
    );

    assert!(html.contains("เงินก้อนที่ลงไปกับรายการนี้ (ถ้ามี)"));
    assert!(html.contains("ค่าใช้จ่ายประจำที่ไม่มีเงินก้อนเริ่มต้น เว้นช่องนี้ได้"));
    assert!(!html.contains("ฐานเงินลงทุน"));
    assert!(html.contains("<small class=\"formal-term\">เงินลงทุน</small>"));
}

#[test]
fn quick_mode_renders_one_ordered_question_per_page() {
    let estimate = calc::QuickEstimate::default();
    let cases = [
        (
            "production",
            "ขั้น 1 จาก 3",
            "ฤดูกาลนี้คาดว่าจะขายทุเรียนได้กี่กิโลกรัม",
            "ผลผลิตที่ขายได้โดยประมาณ",
            "/plans/42",
        ),
        (
            "price",
            "ขั้น 2 จาก 3",
            "คาดว่าจะขายได้ราคาเฉลี่ยกี่บาทต่อกิโลกรัม",
            "ราคาขายเฉลี่ยโดยประมาณ",
            "/plans/42/quick/production",
        ),
        (
            "cost",
            "ขั้น 3 จาก 3",
            "คาดว่าฤดูกาลนี้มีต้นทุนรวมประมาณเท่าไร",
            "ต้นทุนรวมโดยประมาณ",
            "/plans/42/quick/price",
        ),
    ];

    for (step, progress, question, field_label, back) in cases {
        let html = render_quick_question(step, estimate.clone());
        assert!(html.contains(progress), "{step}");
        assert!(html.contains(question), "{step}");
        assert!(html.contains(field_label), "{step}");
        assert_eq!(html.matches("name=\"value\"").count(), 1, "{step}");
        assert!(
            html.contains("aria-describedby=\"quick-value-help\""),
            "{step}"
        );
        assert!(html.contains(back), "{step}");
        assert!(!html.contains("กำไรโดยประมาณ"), "{step}");
    }
}

#[test]
fn quick_result_renders_only_figures_supported_by_three_inputs() {
    let html = render_quick_result(calc::QuickEstimate {
        sellable_yield_kg: Some(rust_decimal::Decimal::new(20_000, 0)),
        average_price_per_kg: Some(rust_decimal::Decimal::new(80, 0)),
        total_cost: Some(rust_decimal::Decimal::new(900_000, 0)),
    });

    for expected in [
        "กำไรโดยประมาณ",
        "700,000 บาท",
        "รายได้โดยประมาณ",
        "1,600,000 บาท",
        "ต้นทุนต่อกิโลกรัม",
        "45.00 บาท/กก.",
        "ราคาขายคุ้มทุน",
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
    assert!(html.contains("ยังไม่ใช้เกรด รายการต้นทุน ROI ภาษี หรือคะแนนสุขภาพสวน"));
}

#[test]
fn incomplete_quick_result_never_invents_or_partially_renders_an_answer() {
    let html = render_quick_result(calc::QuickEstimate {
        sellable_yield_kg: Some(rust_decimal::Decimal::new(20_000, 0)),
        average_price_per_kg: None,
        total_cost: Some(rust_decimal::Decimal::new(900_000, 0)),
    });

    assert!(html.contains("ตอบให้ครบ 3 ข้อก่อนดูผล"));
    assert!(html.contains("/plans/42/quick/price"));
    assert!(!html.contains("hero-value"));
    assert!(!html.contains("700,000.00"));
    assert!(!html.contains("900,000.00"));
}

#[test]
fn detailed_section_is_not_silently_used_while_quick_mode_is_active() {
    let html = Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42/production"));
        let view = view! {
            <Router>
                <PlanSectionView
                    record=quick_record(calc::QuickEstimate::default(), false)
                    section="production".to_owned()
                />
            </Router>
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

    assert!(html.contains("ประมาณการเร็วกำลังใช้งาน"));
    assert!(html.contains("ไม่สลับไปใช้ข้อมูลละเอียดโดยอัตโนมัติ"));
    assert!(!html.contains("บันทึกส่วนนี้"));
}

#[test]
fn quick_url_truthfully_names_when_detailed_mode_is_active() {
    let html = render_detailed_mode_notice();
    assert!(html.contains("แผนละเอียดกำลังใช้งาน"));
    assert!(html.contains("ค่าประมาณการเร็วเดิมยังเก็บไว้"));
    assert!(html.contains("กลับไปดูและแก้แผนละเอียด"));
    assert!(!html.contains("ประมาณการเร็วกำลังใช้งาน"));
}

fn guided_labels(html: &str) -> Vec<&str> {
    let mut labels = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<label class=\"guided-field\"><span>") {
        let after = &rest[start + "<label class=\"guided-field\"><span>".len()..];
        let end = after.find("</span>").expect("primary label closes");
        labels.push(&after[..end]);
        rest = &after[end..];
    }
    labels
}

fn guided_field_chunks(html: &str) -> Vec<&str> {
    html.split("<label class=\"guided-field\">")
        .skip(1)
        .collect()
}

fn headings(html: &str) -> Vec<&str> {
    let mut found = Vec::new();
    for tag in ["h2", "h3", "legend"] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let mut rest = html;
        while let Some(start) = rest.find(&open) {
            let after = &rest[start + open.len()..];
            let end = after.find(&close).expect("heading closes");
            found.push(&after[..end]);
            rest = &after[end..];
        }
    }
    found
}

#[test]
fn every_market_field_states_optionality_and_calculation_effect() {
    let html = render("market", false, PlanForm::from_plan(&calc::Plan::default()));
    let chunks = guided_field_chunks(&html);
    assert_eq!(
        chunks.len(),
        7,
        "all seven market questions are guided fields"
    );
    for chunk in &chunks {
        let help_end = chunk.find("</label>").expect("field closes");
        let field = &chunk[..help_end];
        assert!(field.contains("ไม่บังคับ"), "optionality missing in {field}");
        assert!(
            field.contains("ใช้ในการคำนวณ"),
            "calculation effect missing in {field}"
        );
    }
    assert!(html.contains("มีใครบอกว่าจะรับกี่กิโล"));
    assert!(!html.contains("ความต้องการของตลาด"));
}

#[test]
fn formal_terms_are_secondary_labels_never_the_primary_question() {
    for section in ["market", "production"] {
        let html = render(
            section,
            false,
            PlanForm::from_plan(&calc::workbook_sample()),
        );
        let terms = match section {
            "market" => vec!["ยอดรับซื้อที่คาดไว้"],
            _ => vec!["ผลผลิตขายได้", "สัดส่วนเกรด", "ราคาขายเฉลี่ยถ่วงน้ำหนัก"],
        };
        for term in terms {
            assert!(
                html.contains(&format!("<small class=\"formal-term\">{term}</small>")),
                "{section}: {term} is a secondary label"
            );
            for label in guided_labels(&html) {
                assert!(
                    !label.contains(term),
                    "{section}: {term} leads the question {label}"
                );
            }
            for heading in headings(&html) {
                assert!(
                    !heading.contains(term),
                    "{section}: {term} leads the heading {heading}"
                );
            }
        }
    }
}

#[test]
fn production_offers_both_yield_and_price_branches_and_keeps_the_other() {
    let derived = render(
        "production",
        false,
        PlanForm::from_plan(&calc::workbook_sample()),
    );
    assert!(derived.contains("name=\"yield-source\""));
    assert!(derived.contains("name=\"price-source\""));
    assert!(derived.contains("มีต้นที่ให้ลูกกี่ต้น"));
    assert!(derived.contains("คำนวณได้ประมาณ 19,950.00 กก."));
    assert!(
        derived.contains("≈ 9,975.00 กก."),
        "percent entry shows kilograms beside it"
    );
    assert!(derived.contains("ราคาเฉลี่ยถ่วงน้ำหนักจากทุกเกรด ประมาณ 82.50 บาท/กก."));

    let mut plan = calc::workbook_sample();
    plan.production.yield_source = calc::YieldSource::Direct;
    plan.production.sellable_yield_kg = Some(18_000.into());
    plan.production.price_source = calc::PriceSource::Average;
    plan.production.average_price_per_kg = Some(79.into());
    let direct = render("production", false, PlanForm::from_plan(&plan));
    assert!(direct.contains("กิโลที่คาดว่าจะขายได้ทั้งฤดู"));
    assert!(direct.contains("ขายได้กิโลละเท่าไร"));
    assert!(
        direct.contains("จากข้อมูลต้นทุเรียนที่กรอกไว้ คำนวณได้ประมาณ 19,950.00 กก."),
        "the kept derived figure is shown, not applied"
    );
    assert!(
        direct.contains("จากเกรดที่กรอกไว้ ถ่วงน้ำหนักได้ประมาณ 82.50 บาท/กก."),
        "the kept grade price is shown, not applied"
    );
    assert!(direct.contains("เกรดที่เคยกรอกไว้ยังเก็บอยู่"));
    assert!(
        !direct.contains("มีต้นที่ให้ลูกกี่ต้น"),
        "the unselected branch is not asked"
    );
}

#[test]
fn kilogram_grade_entry_is_unavailable_with_a_reason_until_sellable_is_known() {
    let mut form = PlanForm::from_plan(&calc::Plan::default());
    form.grades.push(Default::default());
    let html = render("production", false, form);
    assert!(html.contains("กรอกเป็นกิโลกรัมได้เมื่อรู้กิโลที่คาดว่าจะขายได้แล้ว"));
    assert!(html.contains("value=\"kilograms\" disabled"));

    let mut form = PlanForm::from_plan(&calc::workbook_sample());
    assert!(form.set_grade_entry(GradeEntry::Kilograms));
    let html = render("production", false, form);
    assert!(!html.contains("กรอกเป็นกิโลกรัมได้เมื่อรู้"));
    assert!(html.contains("เกรดนี้กี่กิโล"));
    assert!(
        html.contains("≈ 50.00% ของกิโลที่คาดว่าจะขายได้"),
        "kilogram entry shows the percentage beside it"
    );
    assert!(html.contains("รวม 19,950.00 จาก 19,950.00 กก."));
}

#[test]
fn empty_cost_sections_offer_unknown_or_confirmed_none_and_rows_hide_the_choice() {
    for (section, name) in [
        ("variable-costs", "variable-cost-state"),
        ("fixed-costs", "fixed-cost-state"),
    ] {
        let empty = render(section, false, PlanForm::from_plan(&calc::Plan::default()));
        assert!(empty.contains(&format!("name=\"{name}\"")), "{section}");
        assert!(empty.contains("ยังไม่รู้ / ข้ามก่อน"), "{section}");
        assert!(empty.contains("ยืนยันว่าไม่มี"), "{section}");
        assert!(empty.contains("ระบบจะนับเป็นศูนย์"), "{section}");

        let filled = render(
            section,
            false,
            PlanForm::from_plan(&calc::workbook_sample()),
        );
        assert!(
            !filled.contains(&format!("name=\"{name}\"")),
            "{section}: rows decide the state"
        );
    }
}

#[test]
fn the_sellable_yield_default_is_disclosed_beside_the_lines_it_fills() {
    let mut plan = calc::workbook_sample();
    let harvest = plan
        .variable_costs
        .iter()
        .position(|line| line.kind == calc::VariableCostKind::HarvestLabor)
        .unwrap();
    let blank_linked = plan
        .variable_costs
        .iter()
        .filter(|line| line.uses_sellable_yield_default())
        .count();
    assert!(
        blank_linked >= 1,
        "the sample leaves at least one linked quantity blank"
    );
    let html = render("variable-costs", false, PlanForm::from_plan(&plan));
    assert_eq!(
        html.matches("quantity-default").count(),
        blank_linked,
        "every blank linked quantity, and only those, shows the applied figure"
    );
    assert!(html.contains("ระบบใช้กิโลที่คาดว่าจะขายได้ 19,950.00 กก. เป็นจำนวนของรายการนี้"));

    assert!(plan.variable_costs[harvest].uses_sellable_yield_default());
    plan.variable_costs[harvest].quantity = Some(10_000.into());
    let html = render("variable-costs", false, PlanForm::from_plan(&plan));
    assert_eq!(
        html.matches("quantity-default").count(),
        blank_linked - 1,
        "an explicit override shows no default note"
    );

    plan.variable_costs[harvest].quantity = None;
    plan.variable_costs[harvest].total_amount = Some(12_000.into());
    let html = render("variable-costs", false, PlanForm::from_plan(&plan));
    assert_eq!(
        html.matches("quantity-default").count(),
        blank_linked - 1,
        "a total-only line takes no default"
    );
    assert!(html.contains("ยอดรวมทั้งฤดู"));
}

#[test]
fn classification_asks_familiar_questions_and_the_expense_lands_intact() {
    let mut form = PlanForm::from_plan(&calc::Plan::default());
    form.unclassified
        .push(web::plan_form::UnclassifiedExpenseForm {
            name: "จ่ายคนขับรถเดือนสาม".into(),
            amount: "50000".into(),
            note: String::new(),
        });
    let html = render("expenses", false, form.clone());
    assert!(html.contains("จำค่าใช้จ่ายได้แต่ยังไม่รู้ว่าเป็นแบบไหน"));
    assert!(html.contains("บอกว่าเป็นแบบไหน"));
    assert!(html.contains("ยังมี 1 รายการที่ยังไม่ได้บอกว่าเป็นแบบไหน จึงยังไม่ถูกนับ"));
    for term in ["ต้นทุนผันแปร", "ต้นทุนคงที่"] {
        for heading in headings(&html) {
            assert!(!heading.contains(term), "{term} leads {heading}");
        }
    }

    let moved = form.classify_expense(
        0,
        web::plan_form::ExpenseClassification::Variable(calc::VariableCostKind::Transport),
    );
    assert!(moved);
    assert!(form.unclassified.is_empty());
    let line = form.variable_costs.last().unwrap();
    assert_eq!(line.name, "จ่ายคนขับรถเดือนสาม");
    assert!(line.total_only);
    assert_eq!(line.total_amount, "50000");
    assert_eq!(line.kind, calc::VariableCostKind::Transport);
    let plan = form.to_plan().unwrap();
    assert_eq!(
        plan.variable_costs.last().unwrap().total_amount,
        Some(50_000.into())
    );

    let mut form = PlanForm::from_plan(&calc::Plan::default());
    form.unclassified
        .push(web::plan_form::UnclassifiedExpenseForm {
            name: "ค่าเช่าที่".into(),
            amount: "36000".into(),
            note: String::new(),
        });
    form.classify_expense(
        0,
        web::plan_form::ExpenseClassification::Fixed(calc::CashKind::Cash),
    );
    assert_eq!(form.fixed_costs[0].amount_per_year, "36000");
    assert_eq!(form.fixed_costs[0].cash_kind, calc::CashKind::Cash);
}

fn plan_with_revenue() -> calc::Plan {
    calc::Plan {
        production: calc::ProductionPlan {
            yield_source: calc::YieldSource::Direct,
            sellable_yield_kg: Some(20_000.into()),
            price_source: calc::PriceSource::Average,
            average_price_per_kg: Some(80.into()),
            ..Default::default()
        },
        variable_cost_state: calc::CostSectionState::ConfirmedNone,
        ..Default::default()
    }
}

fn rent_row() -> web::plan_form::FixedCostForm {
    web::plan_form::FixedCostForm {
        name: "ค่าเช่าที่".into(),
        cash_kind: calc::CashKind::Cash,
        amount_per_year: "36000".into(),
        investment_base: String::new(),
    }
}

fn between<'a>(html: &'a str, from: &str, to: &str) -> &'a str {
    let start = html.find(from).unwrap_or_else(|| panic!("missing {from}"));
    let rest = &html[start..];
    let end = rest
        .find(to)
        .unwrap_or_else(|| panic!("missing {to} after {from}"));
    &rest[..end]
}

#[test]
fn fixed_costs_list_manual_rows_and_included_asset_depreciation_as_two_groups() {
    let mut form = PlanForm::from_plan(&plan_with_revenue());
    form.fixed_costs.push(rent_row());

    let with = render_with_assets("fixed-costs", false, form.clone(), vec![included_asset()]);
    assert!(
        !with.contains("asset-depreciation-note"),
        "the counted note is replaced by the list"
    );
    let manual = between(&with, "cost-group-manual", "cost-group-assets");
    assert!(manual.contains("กรอกเอง"));
    assert!(manual.contains("ค่าเช่าที่"));
    assert!(manual.contains("+ เพิ่มค่าใช้จ่ายประจำ"));
    let assets = between(&with, "cost-group-assets", "fixed-cost-total");
    assert!(assets.contains("ของที่ใช้หลายปี · ค่าเสื่อม"));
    assert!(assets.contains("ต้นทุนไม่ใช่เงินสด"));
    assert!(assets.contains("ระบบน้ำ"));
    assert!(assets.contains("20,000 บาท/ปี"));
    assert!(assets.contains("href=\"/plans/42/assets\""));
    assert!(assets.contains("ไม่ต้องกรอกซ้ำที่นี่"));
    assert!(!assets.contains("<input"), "asset rows are read-only");

    let without = render("fixed-costs", false, form);
    let assets = between(&without, "cost-group-assets", "fixed-cost-total");
    assert!(assets.contains("ยังไม่มีของที่ใช้หลายปีที่รวมในฤดูนี้"));
    assert!(assets.contains("href=\"/plans/42/assets\""));
}

#[test]
fn fixed_costs_total_equals_the_dashboard_fixed_cost() {
    let mut form = PlanForm::from_plan(&plan_with_revenue());
    form.fixed_costs.push(rent_row());
    let plan = form.to_plan().unwrap();
    let dashboard = calc::analyze_with_assets(&plan, &[included_asset()], None)
        .cost
        .fixed_cost
        .unwrap();
    assert_eq!(dashboard, 56_000.into());

    let html = render_with_assets("fixed-costs", false, form.clone(), vec![included_asset()]);
    let total = between(&html, "fixed-cost-total", "</div>");
    assert!(total.contains("รวมค่าใช้จ่ายประจำฤดูนี้"));
    assert!(total.contains("ต้นทุนคงที่รวม"));
    assert!(total.contains("56,000 บาท/ปี"), "{total}");

    let html = render("fixed-costs", false, form);
    let total = between(&html, "fixed-cost-total", "</div>");
    assert!(total.contains("36,000 บาท/ปี"), "{total}");
}

#[test]
fn fixed_costs_question_belongs_to_the_manual_group_and_confirming_makes_asset_only_cost_known() {
    let form = PlanForm::from_plan(&plan_with_revenue());

    let unknown = render_with_assets("fixed-costs", false, form.clone(), vec![included_asset()]);
    let manual = between(&unknown, "cost-group-manual", "cost-group-assets");
    assert!(
        manual.contains("fixed-cost-state"),
        "the question sits in the manual group"
    );
    assert!(manual.contains("นอกจากค่าเสื่อมของที่เลือกไว้ ปีนี้ยังมีอะไรต้องจ่ายแม้ไม่มีทุเรียนขายไหม"));
    assert!(manual.contains("ระบบจะนับเฉพาะค่าเสื่อมของที่เลือกไว้"));
    let assets = between(&unknown, "cost-group-assets", "fixed-cost-total");
    assert!(!assets.contains("fixed-cost-state"));
    let total = between(&unknown, "fixed-cost-total", "</div>");
    assert!(total.contains("ยังไม่รู้"), "{total}");
    assert!(total.contains("ค่าเสื่อม 20,000 บาท/ปี รวมอยู่แล้ว แต่ยอดรวมยังไม่รู้"));
    assert!(
        unknown.contains("ยังคำนวณกำไรสุทธิไม่ได้"),
        "depreciation alone does not make the unknown section known"
    );

    let mut confirmed = form.clone();
    confirmed.fixed_cost_state = calc::CostSectionState::ConfirmedNone;
    let html = render_with_assets("fixed-costs", false, confirmed, vec![included_asset()]);
    let total = between(&html, "fixed-cost-total", "</div>");
    assert!(total.contains("20,000 บาท/ปี"), "{total}");
    assert!(!total.contains("ยังไม่รู้"));
    assert!(
        html.contains("กำไรสุทธิโดยประมาณ 1,580,000 บาท"),
        "the live figure on the page counts the included asset: {html}"
    );

    let mut entered = form;
    entered.fixed_costs.push(rent_row());
    let html = render_with_assets("fixed-costs", false, entered, vec![included_asset()]);
    assert!(
        !html.contains("fixed-cost-state"),
        "rows answer the question"
    );

    let bare = render(
        "fixed-costs",
        false,
        PlanForm::from_plan(&plan_with_revenue()),
    );
    assert!(bare.contains("แม้ไม่มีทุเรียนขาย ปีนี้ยังมีอะไรต้องจ่ายไหม"));
    let total = between(&bare, "fixed-cost-total", "</div>");
    assert!(total.contains("ยังไม่รู้"));
    assert!(!total.contains("รวมอยู่แล้ว"));
}

#[test]
fn hub_names_counted_depreciation_beside_the_missing_fixed_answer() {
    let plan = calc::Plan {
        variable_cost_state: calc::CostSectionState::ConfirmedNone,
        ..Default::default()
    };
    let form = PlanForm::from_plan(&plan);
    assert_eq!(
        form.section_readiness_with_assets("fixed-costs", Some(20_000.into()))
            .label,
        "ค่าเสื่อมของที่เลือกไว้รวมแล้ว ยังต้องกรอกหรือยืนยันว่าไม่มีค่าใช้จ่ายประจำอื่น"
    );
    assert_eq!(
        form.section_readiness_with_assets("fixed-costs", None)
            .label,
        "ยังขาดค่าใช้จ่ายประจำ หรือยืนยันว่าไม่มี"
    );
    let mut confirmed = form;
    confirmed.fixed_cost_state = calc::CostSectionState::ConfirmedNone;
    assert_eq!(
        confirmed
            .section_readiness_with_assets("fixed-costs", Some(20_000.into()))
            .label,
        "ยืนยันแล้วว่ามีเฉพาะค่าเสื่อมของที่เลือกไว้"
    );
}

/// A detailed season whose cost sections are still unknown, with a complete
/// actual draft: the honest case for closing on an incomplete forecast.
fn detailed_record_with_unknown_costs(finalized: bool) -> PlanRecord {
    let plan = calc::Plan {
        production: calc::ProductionPlan {
            yield_source: calc::YieldSource::Direct,
            sellable_yield_kg: Some(20_000.into()),
            price_source: calc::PriceSource::Average,
            average_price_per_kg: Some(80.into()),
            ..Default::default()
        },
        ..Default::default()
    };
    let outcome = calc::ActualOutcome {
        sellable_yield_kg: Some(rust_decimal::Decimal::from(18_000)),
        revenue: Some(rust_decimal::Decimal::from(1_530_000)),
        total_cost: Some(rust_decimal::Decimal::from(990_000)),
        note: String::new(),
    };
    let forecast = finalized.then(|| {
        calc::forecast_metrics_with_assets(
            calc::ForecastMode::Detailed,
            &calc::QuickEstimate::default(),
            &plan,
            &[],
            None,
        )
    });
    PlanRecord {
        id: 42,
        season_year: Some(2569),
        note: String::new(),
        closed: finalized,
        forecast_mode: calc::ForecastMode::Detailed,
        quick_estimate: calc::QuickEstimate::default(),
        starting_capital: None,
        asset_allocations: Vec::new(),
        actual_outcome: Some(ActualOutcomeRecord {
            outcome,
            finalized,
            forecast_mode: finalized.then_some(calc::ForecastMode::Detailed),
            forecast,
        }),
        form: PlanForm::from_plan(&plan),
    }
}

#[test]
fn an_incomplete_forecast_never_blocks_the_close_and_names_what_will_not_compare() {
    let entry = render_actual("entry", detailed_record_with_unknown_costs(false));
    assert!(entry.contains("หลังปิด จะเทียบอะไรกับที่วางไว้ได้บ้าง"));
    assert!(entry.contains("ปิดได้เลย ไม่ต้องกรอกประมาณการให้ครบ แต่ 3 ข้อจะเทียบไม่ได้"));
    assert!(entry.contains("บันทึกและตรวจทาน"));
    assert!(entry.contains("ยังขาด: ค่าใช้จ่ายที่เพิ่มตามการผลิต หรือยืนยันว่าไม่มี"));
    assert!(entry.contains("href=\"/plans/42/variable-costs\""));

    let review = render_actual("review", detailed_record_with_unknown_costs(false));
    let preview = &review[review.find("comparison-preview-rows").expect("preview")..];
    assert_eq!(
        preview.matches("จะเทียบได้<").count(),
        3,
        "kilograms, revenue, price"
    );
    assert_eq!(
        preview.matches("จะเทียบไม่ได้<").count(),
        3,
        "cost, profit, cost per kg"
    );
    assert!(
        review.contains("ยืนยันผลจริงและปิดฤดูกาล"),
        "the close stays available"
    );
    assert!(review.contains("ขายได้จริงกี่กิโล"));
    assert!(review.contains("เหลือหรือขาดจริงเท่าไร"));

    let complete = render_actual("review", actual_record(false));
    assert!(complete.contains("ประมาณการครบ ทุกข้อจะเทียบได้"));
}

#[test]
fn a_season_closed_with_unknown_costs_compares_what_it_froze_and_never_invents_a_zero() {
    let html = render_actual("comparison", detailed_record_with_unknown_costs(true));
    assert!(html.contains("แหล่งประมาณการ: แผนละเอียด"));
    assert!(html.contains("ต่ำกว่าประมาณการ 2,000.00 กก."));
    assert!(
        html.contains("ต่ำกว่าประมาณการ 70,000 บาท"),
        "revenue compares"
    );
    assert_eq!(
        html.matches("เทียบไม่ได้ ตอนปิดยังไม่มีประมาณการตัวนี้").count(),
        3,
        "cost, profit, and cost per kg have no frozen forecast"
    );
    assert_eq!(html.matches("ไม่มีตอนปิด").count(), 3);
    assert!(!html.contains("ประมาณการ</dt><dd>0.00"));
    assert!(html.contains("เงินที่จ่ายไปทั้งหมด"));
    assert!(html.contains("<small class=\"formal-term\">ต้นทุนรวม</small>"));
}

// The deductions section: nothing pre-filled, suggestions offered for the
// name only, every string from DESIGN.md, and a closed season renders the
// lines as text with no controls.
#[test]
fn the_deductions_section_offers_names_prefills_nothing_and_locks_when_closed() {
    let mut none = calc::workbook_sample();
    none.tax_deductions.clear();
    let html = render("tax-deductions", false, PlanForm::from_plan(&none));
    for expected in [
        "ปีนี้มีอะไรลดหย่อนภาษีได้บ้าง",
        "ระบบไม่ใส่ให้เอง แม้แต่ค่าลดหย่อนส่วนตัว",
        "กรอกแล้วได้อะไร ภาษีโดยประมาณจะหักรายการเหล่านี้ออกก่อนคิด",
        "+ เพิ่มรายการลดหย่อน",
        "<datalist id=\"deduction-names\">",
        "<option value=\"ค่าลดหย่อนส่วนตัว\"",
        "<option value=\"เงินบริจาค\"",
        "ดูภาษีโดยประมาณ",
    ] {
        assert!(html.contains(expected), "missing {expected} in {html}");
    }
    assert!(!html.contains("60,000"), "nothing is pre-filled: {html}");
    assert!(
        !html.contains("รวมลดหย่อน"),
        "no total without a line: {html}"
    );

    let open = render(
        "tax-deductions",
        false,
        PlanForm::from_plan(&calc::workbook_sample()),
    );
    for expected in [
        "ลดหย่อนจากอะไร",
        "list=\"deduction-names\"",
        "จำนวนเงินที่ลดหย่อนได้ปีนี้",
        "กรอกตามที่คุณมีสิทธิ์จริง เช่น ค่าลดหย่อนส่วนตัว 60,000",
        "ระบบไม่ตรวจเพดานให้",
        "ลบรายการนี้",
        "รวมลดหย่อน",
        "60,000 บาท",
    ] {
        assert!(open.contains(expected), "missing {expected} in {open}");
    }

    let closed = render(
        "tax-deductions",
        true,
        PlanForm::from_plan(&calc::workbook_sample()),
    );
    assert!(closed.contains("ค่าลดหย่อนส่วนตัว"), "{closed}");
    assert!(!closed.contains("+ เพิ่มรายการลดหย่อน"), "{closed}");
    assert!(!closed.contains("ลบรายการนี้"), "{closed}");
    assert!(
        !closed.contains("type=\"text\""),
        "a closed season has no controls: {closed}"
    );

    let closed_empty = render("tax-deductions", true, PlanForm::from_plan(&none));
    assert!(
        closed_empty.contains("ไม่ได้กรอกรายการลดหย่อนไว้ ภาษีของฤดูนี้จึงคิดโดยยังไม่หักลดหย่อน"),
        "{closed_empty}"
    );
}
