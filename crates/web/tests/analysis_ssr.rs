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
    plan_ui::PlanHub,
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
            starting_capital: None,
            asset_allocations: Vec::new(),
            actual_outcome: None,
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

fn dashboard_for(plan: &Plan) -> String {
    dashboard(PlanForm::from_plan(plan))
}

fn render_hub(form: PlanForm) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new("/plans/42"));
        let record = PlanRecord {
            id: 42,
            season_year: Some(2569),
            note: "ปีทดสอบ".into(),
            closed: false,
            forecast_mode: calc::ForecastMode::Detailed,
            quick_estimate: calc::QuickEstimate::default(),
            starting_capital: None,
            asset_allocations: Vec::new(),
            actual_outcome: None,
            form,
        };
        let view = view! { <Router><PlanHub record/></Router> };
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

fn analysis(form: PlanForm) -> String {
    render_with(form, true)
}

fn sample_form() -> PlanForm {
    PlanForm::from_plan(&workbook_sample())
}

fn sample_analysis() -> Analysis {
    calc::analyze(&workbook_sample())
}

/// An amount of money as the page shows it since revision 0.12: whole baht.
fn whole(value: Decimal) -> String {
    let plain = format!(
        "{:.0}",
        value.round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
    );
    let (sign, digits) = plain
        .strip_prefix('-')
        .map_or(("", plain.as_str()), |digits| ("-", digits));
    let mut grouped = String::new();
    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    format!("{sign}{}", grouped.chars().rev().collect::<String>())
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
            html.contains(&money(value)) || html.contains(&format!("{} บาท<", whole(value))),
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
fn an_empty_plan_names_the_missing_question_instead_of_showing_a_figure() {
    let html = dashboard(PlanForm::from_plan(&Plan::default()));

    assert!(!html.contains("hero-value"));
    assert!(html.contains("ยังบอกไม่ได้"));
    assert!(html.contains("ยังขาด: ปีนี้จะขายได้กี่กิโล"));
    assert!(html.contains("href=\"/plans/42/production\""));
    // Optional results are named as optional, with their unlock, never as
    // something missing; nothing is called incomplete.
    assert!(html.contains("เพิ่มได้: ยอดที่ผู้ซื้อคุยว่าจะรับ"));
    assert!(html.contains("เพิ่มได้: แบบประเมินสวน 12 ข้อ"));
    assert!(!html.contains("ยังไม่ครบ"));
    assert!(!html.contains("ยังคำนวณไม่ได้"));
    // No figure row shows a number for an absent input.
    assert!(!html.contains("0 บาท"));
}

/// The six decisions, in the order the list renders them, as one string per
/// row so two pages can be compared.
fn decision_rows(html: &str) -> Vec<String> {
    let start = html.find("decision-rows").expect("a decision list");
    let rest = &html[start..];
    let end = rest.find("</ul>").expect("the list closes");
    rest[..end]
        .split("<li")
        .skip(1)
        .map(|row| {
            let mut text = String::new();
            let mut in_tag = false;
            for ch in row.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => text.push(ch),
                    _ => {}
                }
            }
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect()
}

#[test]
fn readiness_is_named_by_decision_and_agrees_across_hub_dashboard_and_analysis() {
    // Sellable kilograms and a price are known; costs are not, and the
    // owner has said nothing about buyers or the self-assessment.
    let plan = Plan {
        production: calc::ProductionPlan {
            yield_source: calc::YieldSource::Direct,
            sellable_yield_kg: Some(20_000.into()),
            price_source: calc::PriceSource::Average,
            average_price_per_kg: Some(80.into()),
            ..Default::default()
        },
        ..Default::default()
    };
    let form = PlanForm::from_plan(&plan);
    let hub = render_hub(form.clone());
    let dashboard = dashboard(form.clone());
    let analysis = analysis(form);

    let rows = decision_rows(&dashboard);
    assert_eq!(rows.len(), 6, "{rows:?}");
    assert_eq!(rows, decision_rows(&hub));
    // The analysis page says the same thing in one line that links to the
    // hub: how many can be answered, and what is still missing, once.
    assert!(!analysis.contains("decision-rows"), "{analysis}");
    assert!(analysis.contains("ตอบได้ 1 จาก 6 คำถาม"), "{analysis}");
    assert!(
        analysis.contains("ยังขาด: ค่าใช้จ่ายที่เพิ่มตามการผลิต หรือยืนยันว่าไม่มี<"),
        "{analysis}"
    );
    assert_eq!(analysis.matches("ยังขาด:").count(), 1, "named once");
    let summary = analysis.find("decision-summary").expect("one-line summary");
    assert!(
        analysis[..summary].rfind("<a href=\"/plans/42\"").is_some(),
        "links to the hub: {analysis}"
    );

    assert!(rows[0].contains("ปีนี้จะเหลือกำไรเท่าไร"));
    assert!(rows[0].contains("กำไรสุทธิ"));
    assert!(rows[0].contains("ยังขาด: ค่าใช้จ่ายที่เพิ่มตามการผลิต หรือยืนยันว่าไม่มี"));
    assert!(rows[1].contains("เพิ่มได้: ยอดที่ผู้ซื้อคุยว่าจะรับ"));
    assert!(rows[2].contains("ยังขาด: ค่าใช้จ่ายที่เพิ่มตามการผลิต"));
    assert!(rows[3].contains("ยังขาด: ค่าใช้จ่ายที่เพิ่มตามการผลิต"));
    assert!(rows[4].contains("เพิ่มได้: แบบประเมินสวน 12 ข้อ"));
    assert!(
        rows[5].contains("ดูได้แล้ว"),
        "closing never waits on a forecast: {}",
        rows[5]
    );
    for page in [&hub, &dashboard, &analysis] {
        assert!(page.contains("href=\"/plans/42/variable-costs\""));
    }

    // With a complete first estimate the dashboard drops the list for the
    // one line too; the hub is the only page that keeps all six rows.
    let ready = dashboard_for(&workbook_sample());
    assert!(!ready.contains("decision-rows"), "{ready}");
    assert!(ready.contains("ตอบได้ 6 จาก 6 คำถาม"), "{ready}");
    assert!(!ready.contains("ยังขาด:"), "{ready}");
}

/// A formal term may appear only as the secondary label under the owner's
/// words, never as the heading or the leading text of a figure.
fn assert_formal_terms_lead_nowhere(html: &str, terms: &[&str]) {
    for term in terms {
        let mut found = 0;
        for (index, _) in html.match_indices(term) {
            found += 1;
            let before = &html[..index];
            let tag_start = before.rfind('<').expect("inside markup");
            let tag = &before[tag_start..];
            let leads = tag.starts_with("<h1")
                || tag.starts_with("<h2")
                || tag.starts_with("<h3")
                || tag.starts_with("<strong")
                || tag.starts_with("<span class=\"figure-words\"")
                || tag.starts_with("<span class=\"decision-question\"");
            assert!(
                !leads,
                "{term} leads a figure or heading: …{}",
                &html[index.saturating_sub(80)..index + term.len()]
            );
        }
        assert!(found > 0, "{term} is no longer taught anywhere on the page");
    }
}

#[test]
fn dashboard_leads_with_familiar_wording_and_keeps_formal_terms_secondary() {
    let html = dashboard(sample_form());
    for plain in [
        "เหลือหลังหักค่าใช้จ่ายทั้งหมด",
        "ต้นทุนต่อ 1 กก. ที่ขาย",
        "ต้องขายอย่างน้อยกี่กิโลจึงไม่ขาดทุน",
        "กำไรเทียบกับเงินก้อนที่ลงไป",
        "อีกกี่ปีเงินก้อนจะกลับมาครบ",
        "ขายได้ทั้งฤดู",
        "เงินสดที่เหลือจากฤดูนี้",
        "ราคาเฉลี่ยของทุกเกรดรวมกัน",
        "เงินเหลือต่อ 1 กก. ก่อนจ่ายค่าใช้จ่ายประจำ",
        "ขายได้เกินขั้นต่ำที่ไม่ขาดทุนกี่กิโล",
        "ของที่มีเทียบกับที่ผู้ซื้อคุยไว้",
        "คะแนนที่ประเมินสวนเอง",
    ] {
        assert!(html.contains(plain), "{plain} is missing");
    }
    assert_formal_terms_lead_nowhere(
        &html,
        &[
            "กำไรสุทธิ",
            "จุดคุ้มทุน",
            "ROI",
            "ระยะคืนทุน",
            "กระแสเงินสด",
            "ราคาขายเฉลี่ยถ่วงน้ำหนัก",
            "ส่วนเกินต่อหน่วย",
            "ส่วนเผื่อความปลอดภัย",
            "คะแนนสุขภาพธุรกิจ",
        ],
    );
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

    assert!(html.contains("ตั้งเป้าในขั้นสูง"));
    assert!(html.contains("/plans/42/targets"));
    assert!(!html.contains("ถึงเป้า"));
    assert!(!html.contains("ต้องปรับปรุง"));
}

#[test]
fn missing_physical_quantities_are_quiet_and_do_not_render_incomplete_kpis() {
    let mut plan = workbook_sample();
    plan.variable_costs.retain(|line| {
        !matches!(
            line.kind,
            calc::VariableCostKind::OrchardLabor
                | calc::VariableCostKind::Fertilizer
                | calc::VariableCostKind::Water
                | calc::VariableCostKind::Electricity
        )
    });
    let html = analysis(PlanForm::from_plan(&plan));

    for hidden in ["ผลผลิตต่อวันแรงงาน", "ผลผลิตต่อปุ๋ย", "ผลผลิตต่อน้ำ", "ผลผลิตต่อไฟฟ้า"]
    {
        assert!(
            !html.contains(hidden),
            "{hidden} should stay quiet without a quantity"
        );
    }
    assert!(html.contains("ตัวที่ไม่ได้กรอกไม่ถูกนับว่าขาด"));
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

    // Each rule is a plain question first and its formal name second.
    for (question, formal) in [
        ("สัดส่วนเกรดรวมกันครบ 100% ไหม", "สัดส่วนเกรดรวมได้ 100%"),
        ("มีกิโลที่จะขายได้มากกว่าศูนย์ไหม", "ผลผลิตขายได้เป็นบวก"),
        (
            "ขายได้กิโลละมากกว่าที่จ่ายต่อกิโลไหม",
            "ราคาขายสูงกว่าต้นทุนผันแปรต่อหน่วย",
        ),
        ("มีเงินก้อนที่ลงไปอย่างน้อย 1 รายการไหม", "ฐานเงินลงทุนเป็นบวก"),
        ("ยอดรวมค่าใช้จ่ายมาจากรายการที่กรอกไหม", "ต้นทุนรวมเชื่อมกับรายการ"),
        ("ตอบแบบประเมินสวนครบ 12 ข้อไหม", "คำตอบสุขภาพธุรกิจครบ"),
    ] {
        assert!(html.contains(question), "{question} is missing");
        assert!(
            html.contains(&format!("<small class=\"formal-term\">{formal}")),
            "{formal} is not the secondary label"
        );
    }
    // A rule's answer is an observation about the entered figures, never a
    // verdict about the orchard.
    assert!(!html.contains("ต้องแก้"));
    assert!(!html.contains("พร้อมใช้ตัดสินใจ"));
    assert!(html.contains("/plans/42/production"));
    assert!(html.contains("/plans/42/variable-costs"));
    assert!(html.contains("/plans/42/fixed-costs"));

    let readiness = match sample_analysis().checks.overall {
        calc::Readiness::Ready => "ไม่พบข้อที่ควรตรวจ",
        calc::Readiness::NeedsReview => "มีข้อที่ควรตรวจ",
    };
    assert!(
        html.contains(readiness),
        "the rendered readiness does not match the engine"
    );
}

#[test]
fn both_tax_methods_render_after_a_prominent_non_recommendation_disclaimer() {
    let html = analysis(sample_form());
    let tax = sample_analysis().tax;

    assert!(html.contains("หักค่าใช้จ่ายตามจริง"));
    assert!(html.contains("หักค่าใช้จ่ายแบบเหมา"));
    assert!(html.contains("ไม่ใช่คำแนะนำทางภาษี"));

    let actual = tax.actual_expense.estimated_tax.expect("actual method");
    let flat = tax.flat_sixty_percent.estimated_tax.expect("flat method");
    assert!(html.contains(&whole(actual)));
    assert!(html.contains(&whole(flat)));
    assert!(!html.contains("เสียน้อยกว่า"));
    assert!(!html.contains("tax-method cheaper"));
    let disclaimer = html.find("ยังไม่ได้ยืนยันแหล่งกฎหมาย").expect("disclaimer");
    let first_figure = html.find(&whole(actual)).expect("first tax figure");
    assert!(
        disclaimer < first_figure,
        "the boundary appears before figures"
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
                html.contains(&whole(cell)),
                "the matrix is missing {}",
                whole(cell)
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
        html.contains(&whole(centre)),
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

/// Result surfaces make arithmetic observations. They never tell the owner
/// what to do, what caused a figure, or what is wrong with the orchard, and
/// every ⓘ sheet links to the page holding the inputs it was computed from.
#[test]
fn result_explanations_observe_arithmetic_and_link_to_their_inputs() {
    let forbidden = [
        "ไม่ควร",
        "ต้องเร่ง",
        "สาเหตุ",
        "แนะนำให้",
        "ควรทำ",
        "ล้มได้",
        "มักแปลว่า",
        "ทำได้ทางเดียว",
        "ต้องปรับปรุง",
        "ต้องแก้",
        "พร้อมใช้ตัดสินใจ",
    ];
    for (name, html) in [
        ("dashboard", dashboard(sample_form())),
        ("analysis", analysis(sample_form())),
        ("hub", render_hub(sample_form())),
    ] {
        // The tax disclaimer's "ไม่ควรใช้เลือกวิธียื่น" is the boundary of
        // the estimate, not advice about the orchard, and stays.
        let scanned = html.replace(web::explanations::TAX_DISCLAIMER, "");
        for phrase in forbidden {
            assert!(
                !scanned.contains(phrase),
                "{name} still says {phrase}: …{}…",
                excerpt(&scanned, phrase)
            );
        }
        let sheets = html.matches("class=\"figure-explanation\"").count();
        let links = html.matches("explanation-input").count();
        if sheets > 0 {
            assert_eq!(sheets, links, "{name}: every ⓘ sheet links to its inputs");
            assert!(html.contains("href=\"/plans/42/"));
        }
    }
}

fn excerpt(html: &str, phrase: &str) -> String {
    let at = html.find(phrase).unwrap_or(0);
    let start = at.saturating_sub(60);
    html[start..(at + phrase.len() + 30).min(html.len())]
        .chars()
        .collect()
}

// The tax screen reads the owner's deduction lines. With none entered it
// deducts nothing and says so in the same breath as the figure, with the way
// to the section; with lines it says how many and how much; when the lines
// exceed income after expense the zero is explained, never graded.
#[test]
fn the_tax_screen_states_the_deduction_state_beside_the_figures() {
    let mut none = workbook_sample();
    none.tax_deductions.clear();
    let html = analysis(PlanForm::from_plan(&none));
    assert!(
        html.contains("ยังไม่ได้หักลดหย่อน ตัวเลขนี้จึงสูงกว่าภาษีจริง"),
        "{html}"
    );
    assert!(html.contains("href=\"/plans/42/tax-deductions\""), "{html}");
    assert!(html.contains(">กรอกลดหย่อน<"), "{html}");
    assert!(html.contains("หักลดหย่อนอีก"), "{html}");
    assert!(!html.contains("หักส่วนตัวอีก"), "{html}");
    // 1,645,875 − 811,275 with nothing deducted.
    assert!(html.contains(&whole(Decimal::from(834_600))), "{html}");
    assert!(!html.contains("ลดหย่อนมากกว่าเงินได้"), "{html}");

    let with_lines = analysis(sample_form());
    assert!(
        with_lines.contains("หักลดหย่อนแล้ว 1 รายการ รวม 60,000 บาท"),
        "{with_lines}"
    );
    assert!(with_lines.contains(">ดูหรือแก้<"), "{with_lines}");
    assert!(
        with_lines.contains(&whole(Decimal::from(774_600))),
        "{with_lines}"
    );

    let mut exceeding = workbook_sample();
    exceeding.tax_deductions = vec![calc::TaxDeductionLine {
        name: "รวมทุกอย่าง".into(),
        amount: Decimal::from(900_000),
    }];
    let html = analysis(PlanForm::from_plan(&exceeding));
    assert!(
        html.contains("ลดหย่อนมากกว่าเงินได้หลังหักค่าใช้จ่าย จึงไม่มีภาษี"),
        "{html}"
    );
    assert!(
        html.contains("หักลดหย่อนแล้ว 1 รายการ รวม 900,000 บาท"),
        "{html}"
    );
}

#[test]
fn the_hub_row_for_deductions_names_its_state_and_never_blocks_readiness() {
    let mut none = workbook_sample();
    none.tax_deductions.clear();
    let html = render_hub(PlanForm::from_plan(&none));
    assert!(html.contains("ปีนี้มีอะไรลดหย่อนภาษีได้บ้าง"), "{html}");
    assert!(html.contains("ยังไม่ได้กรอก · ภาษีคิดโดยยังไม่หักลดหย่อน"), "{html}");

    let html = render_hub(sample_form());
    assert!(html.contains("กรอกแล้ว 1 รายการ · รวม 60,000 บาท"), "{html}");
}
