#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use web::{
    plan_form::PlanForm,
    plan_ui::{
        ActualCloseView, ActualComparisonView, ActualReviewView, DemoPage, PlanHub,
        PlanSectionView, QuickQuestionView, QuickResultView, SeasonHistoryView,
    },
    plans::{ActualOutcomeRecord, PlanRecord, SeasonHistoryItem},
};

const SECTIONS: [(&str, &str); 6] = [
    ("market", "ตลาด"),
    ("production", "ผลผลิตและเกรด"),
    ("variable-costs", "ต้นทุนผันแปร"),
    ("fixed-costs", "ต้นทุนคงที่"),
    ("targets", "เป้าหมาย"),
    ("health", "สุขภาพสวน"),
];

fn render(section: &str, closed: bool, form: PlanForm) -> String {
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
                        asset_allocations: Vec::new(),
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
    assert!(html.contains("1,530,000.00 บาท"));
    assert!(html.contains("990,000.00 บาท"));
    assert!(html.contains("540,000.00 บาท"));
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
    assert!(html.contains("สูงกว่าประมาณการ 90,000.00 บาท"));
    assert!(html.contains("ต่ำกว่าประมาณการ 160,000.00 บาท"));
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
    assert!(!html.contains("0.00 บาท"));
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
    assert!(html.contains("ต้นทุนรวม สูงกว่าฤดูกาลก่อน 10.00%"));
    assert!(html.contains("ผลผลิตที่ขายได้ ต่ำกว่าฤดูกาลก่อน 10.00%"));
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
    let html = Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42"));
        let view = view! {
            <Router>
                <PlanHub record=PlanRecord {
                    id: 42,
                    season_year: Some(2569),
                    note: String::new(),
                    closed: false,
                    forecast_mode: calc::ForecastMode::Detailed,
                    quick_estimate: calc::QuickEstimate::default(),
                    starting_capital: None,
                    asset_allocations: Vec::new(),
                    actual_outcome: None,
                    form: PlanForm::from_plan(&calc::workbook_sample()),
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
    });

    assert!(html.contains("การวางแผนขั้นสูง (ไม่บังคับ)"));
    assert!(html.contains("เป้าหมาย KPI"));
    assert!(html.contains("ค่าที่ตั้งไว้เดิมยังอยู่และแก้ได้ที่นี่"));
    assert!(html.contains("/plans/42/targets"));
    assert_eq!(html.matches("ตัวเลขเปรียบเทียบที่เจ้าของกำหนด").count(), 0);
}

#[test]
fn all_six_input_routes_render_an_empty_editable_state() {
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
fn all_six_input_routes_render_closed_values_without_edit_controls() {
    for (section, title) in SECTIONS {
        let html = render(section, true, PlanForm::from_plan(&calc::workbook_sample()));
        assert!(html.contains(title), "{section}");
        assert!(html.contains("ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"), "{section}");
        assert!(!html.contains("บันทึกส่วนนี้"), "{section}");
        assert!(html.contains("readonly-value"), "{section}");
    }
}

#[test]
fn fixed_costs_explain_that_investment_is_optional_per_item() {
    let html = render(
        "fixed-costs",
        false,
        PlanForm::from_plan(&calc::workbook_sample()),
    );

    assert!(html.contains("เงินที่ลงทุนกับรายการนี้ (ถ้ามี)"));
    assert!(html.contains("ค่าใช้จ่ายประจำที่ไม่มีเงินก้อนเริ่มต้น เว้นช่องนี้ได้"));
    assert!(!html.contains("ฐานเงินลงทุน"));
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
        "700,000.00 บาท",
        "รายได้โดยประมาณ",
        "1,600,000.00 บาท",
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
