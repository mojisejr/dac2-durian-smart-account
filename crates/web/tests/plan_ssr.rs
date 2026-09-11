#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use web::{
    plan_form::PlanForm,
    plan_ui::{DemoPage, PlanSectionView, QuickQuestionView, QuickResultView},
    plans::PlanRecord,
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

#[test]
fn demonstration_is_an_editable_browser_surface_not_a_persisting_form() {
    let html = render_demo();
    assert!(html.contains("ลองก่อน โดยไม่บันทึก"));
    assert!(html.contains("คืนค่าตัวอย่าง"));
    assert!(html.contains("สร้างฤดูกาลของฉัน"));
    assert!(!html.contains("<form"));
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
