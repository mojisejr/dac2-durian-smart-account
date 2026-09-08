#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use web::{plan_form::PlanForm, plan_ui::PlanSectionView, plans::PlanRecord};

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
                    record=PlanRecord { id: 42, closed, form: form.clone() }
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
