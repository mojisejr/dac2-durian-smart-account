//! Every derived surface the calculation engine already produces, rendered.
//!
//! This module adds no arithmetic. It reads an [`calc::Analysis`] and decides
//! only how a figure is shown, what it is called, and when a figure must be
//! withheld because the input it needs is absent.

use calc::{
    Analysis, CheckKind, CheckResult, CheckStatus, HealthDimension, HealthStatus, KpiKind,
    KpiResult, KpiVerdict, Readiness, TaxMethodAnalysis,
};
use leptos::prelude::*;
use leptos_router::components::A;
use rust_decimal::Decimal;

use crate::{
    explanations::{self, Explanation},
    plan_form::{Decision, DecisionReadiness, DecisionState, PlanForm, ReadinessTone},
    plan_ui::{BottomNav, DecisionList, NavSection, baht_amount, money},
    plans::PlanRecord,
};

/// Shown wherever a figure cannot be computed from what the owner has entered.
const ABSENT: &str = "ยังไม่มีข้อมูล";
const SECTION_TITLES: [(&str, &str); 5] = [
    ("market", "ตลาด"),
    ("production", "ผลผลิตและเกรด"),
    ("variable-costs", "ต้นทุนผันแปร"),
    ("fixed-costs", "ต้นทุนคงที่"),
    ("health", "สุขภาพสวน"),
];

// ---------------------------------------------------------------- formatting

fn baht(value: Option<Decimal>) -> String {
    value.map_or_else(
        || ABSENT.into(),
        |value| format!("{} บาท", baht_amount(value)),
    )
}

fn with_unit(value: Option<Decimal>, unit: &str) -> String {
    value.map_or_else(|| ABSENT.into(), |value| format!("{} {unit}", money(value)))
}

fn percent(value: Option<Decimal>) -> String {
    value.map_or_else(
        || ABSENT.into(),
        |value| format!("{}%", money(value * Decimal::ONE_HUNDRED)),
    )
}

fn score(value: Option<Decimal>) -> String {
    value.map_or_else(|| ABSENT.into(), |value| format!("{}/5", money(value)))
}

fn has_positive_investment(value: Option<Decimal>) -> bool {
    value.is_some_and(|value| value > Decimal::ZERO)
}

fn roi_figure(analysis: &Analysis) -> Result<String, &'static str> {
    match analysis.business.roi {
        Some(value) => Ok(percent(Some(value))),
        None if !has_positive_investment(analysis.cost.investment_base) => {
            Err("กรอกเงินลงทุนอย่างน้อย 1 รายการ")
        }
        None => Err(ABSENT),
    }
}

fn payback_figure(analysis: &Analysis) -> Result<String, &'static str> {
    match analysis.business.payback_years {
        Some(value) => Ok(with_unit(Some(value), "ปี")),
        None if !has_positive_investment(analysis.cost.investment_base) => {
            Err("กรอกเงินลงทุนอย่างน้อย 1 รายการ")
        }
        None if analysis
            .business
            .operating_cash_flow
            .is_some_and(|value| value <= Decimal::ZERO) =>
        {
            Err("ยังคืนทุนไม่ได้ เพราะกระแสเงินสดไม่เป็นบวก")
        }
        None => Err(ABSENT),
    }
}

// -------------------------------------------------------------------- labels

const fn kpi_label(kind: KpiKind) -> (&'static str, &'static str) {
    match kind {
        KpiKind::YieldPerRai => ("ผลผลิตต่อไร่", "กก./ไร่"),
        KpiKind::YieldPerTree => ("ผลผลิตต่อต้น", "กก./ต้น"),
        KpiKind::YieldPerLaborDay => ("ผลผลิตต่อวันแรงงาน", "กก./วัน"),
        KpiKind::YieldPerFertilizerKg => ("ผลผลิตต่อปุ๋ย", "กก./กก."),
        KpiKind::YieldPerWaterCubicMeter => ("ผลผลิตต่อน้ำ", "กก./ลบ.ม."),
        KpiKind::YieldPerKwh => ("ผลผลิตต่อไฟฟ้า", "กก./kWh"),
        KpiKind::QualityGradeShare => ("สัดส่วนเกรดคุณภาพ", "%"),
        KpiKind::LossShare => ("สัดส่วนสูญเสีย", "%"),
        KpiKind::CostPerKg => ("ต้นทุนต่อกิโลกรัม", "บาท/กก."),
    }
}

const fn kpi_is_share(kind: KpiKind) -> bool {
    matches!(kind, KpiKind::QualityGradeShare | KpiKind::LossShare)
}

/// The rule, what it means in Thai, and the input that would settle it.
/// The plain question, the formal name of the rule, and the page that
/// would change it.
const fn check_label(kind: CheckKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        CheckKind::GradeSharesTotalOne => (
            "สัดส่วนเกรดรวมกันครบ 100% ไหม",
            "สัดส่วนเกรดรวมได้ 100%",
            "production",
        ),
        CheckKind::SellableYieldPositive => {
            ("มีกิโลที่จะขายได้มากกว่าศูนย์ไหม", "ผลผลิตขายได้เป็นบวก", "production")
        }
        CheckKind::PriceAboveVariableCost => (
            "ขายได้กิโลละมากกว่าที่จ่ายต่อกิโลไหม",
            "ราคาขายสูงกว่าต้นทุนผันแปรต่อหน่วย",
            "variable-costs",
        ),
        CheckKind::InvestmentPositive => (
            "มีเงินก้อนที่ลงไปอย่างน้อย 1 รายการไหม",
            "ฐานเงินลงทุนเป็นบวก",
            "fixed-costs",
        ),
        CheckKind::TotalCostLinked => (
            "ยอดรวมค่าใช้จ่ายมาจากรายการที่กรอกไหม",
            "ต้นทุนรวมเชื่อมกับรายการ",
            "variable-costs",
        ),
        CheckKind::HealthAnswersComplete => {
            ("ตอบแบบประเมินสวนครบ 12 ข้อไหม", "คำตอบสุขภาพธุรกิจครบ", "health")
        }
    }
}

const fn check_status_label(status: CheckStatus) -> (&'static str, &'static str) {
    match status {
        CheckStatus::Ok => ("ใช่", "status good"),
        CheckStatus::Warning => ("ยังไม่ใช่ ควรตรวจ", "status warning"),
        CheckStatus::NeedsCheck => ("ยังตรวจไม่ได้", "status muted"),
    }
}

const fn dimension_label(dimension: HealthDimension) -> &'static str {
    match dimension {
        HealthDimension::Finance => "การเงิน",
        HealthDimension::Production => "ผลผลิต",
        HealthDimension::Market => "ตลาด",
        HealthDimension::Resources => "ทรัพยากร",
        HealthDimension::People => "คน",
        HealthDimension::Resilience => "ความทนทาน",
    }
}

const fn health_status_label(status: HealthStatus) -> (&'static str, &'static str) {
    match status {
        HealthStatus::Strong => ("ให้คะแนนตัวเองสูง", "status good"),
        HealthStatus::Watch => ("ให้คะแนนตัวเองปานกลาง", "status muted"),
        HealthStatus::ImproveUrgently => ("ให้คะแนนตัวเองต่ำ", "status warning"),
    }
}

// ---------------------------------------------------------------- components

/// The `ⓘ` affordance: four questions, always in the same order, in a sheet.
///
/// Measured at 320 pixels wide, every explanation is taller than half the
/// screen and six are taller than the whole of a short one, so this is not
/// tooltip-sized content and never was. The sheet rises from the bottom, leaves
/// the figure it explains visible above it, and closes by its own button rather
/// than by finding the same small icon again.
#[component]
pub fn Explain(
    explanation: Explanation,
    label: String,
    /// The season whose inputs the sheet links to; `None` on the sample.
    #[prop(default = None)]
    plan_id: Option<i64>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let title = label.clone();
    let sheet_label = label.clone();
    let (input_section, input_label) = explanation.input;
    view! {
        <details class="figure-explanation" open=move || open.get()>
            <summary
                aria-label=format!("อธิบาย{label}")
                on:click=move |event| {
                    // Take the toggle over so the close button and the backdrop
                    // can drive it too. Without script the browser's own toggle
                    // still runs and the icon remains the way back out.
                    event.prevent_default();
                    open.update(|value| *value = !*value);
                }
            >"ⓘ"</summary>
            <span class="sheet-backdrop" on:click=move |_| open.set(false)></span>
            <div class="sheet" role="group" aria-label=format!("คำอธิบาย {sheet_label}")>
                <div class="sheet-head"><h3 class="sheet-title">{title}</h3></div>
                <div class="sheet-body">
                    <h3>"คืออะไร"</h3><p>{explanation.what}</p>
                    <h3>"ใช้ยังไง"</h3><p>{explanation.how}</p>
                    <h3>"ทำไมต้องมี"</h3><p>{explanation.why}</p>
                    <h3>"ไม่ใส่ได้ไหม"</h3><p>{explanation.missing}</p>
                    {plan_id.map(|id| view! {
                        <A attr:class="text-button explanation-input" href=format!("/plans/{id}/{input_section}")>{input_label}</A>
                    })}
                </div>
                <button class="sheet-close" type="button" on:click=move |_| open.set(false)>"ปิด"</button>
            </div>
        </details>
    }
}

/// One figure row: the plain question first, the accounting term under it,
/// and the value or the reason it is absent on the right.
#[component]
fn Figure(
    label: &'static str,
    #[prop(optional)] formal_term: Option<&'static str>,
    value: String,
    #[prop(optional)] explanation: Option<Explanation>,
    #[prop(default = None)] absent_reason: Option<String>,
    #[prop(default = None)] plan_id: Option<i64>,
) -> impl IntoView {
    let explain_label = label;
    view! {
        <div class="figure-row">
            <span class="figure-label">
                <span class="figure-words">
                    {label}
                    {formal_term.map(|term| view! { <small class="formal-term">{term}</small> })}
                </span>
                {explanation.map(|explanation| view! {
                    <Explain explanation label=explain_label.to_owned() plan_id/>
                })}
            </span>
            {match absent_reason {
                Some(reason) => view! { <span class="figure-value figure-missing">{reason}</span> }.into_any(),
                None => view! { <strong class="figure-value">{value}</strong> }.into_any(),
            }}
        </div>
    }
}

/// The reason a figure is absent, in the words of the decision it belongs to.
fn absent_because(decisions: &[DecisionReadiness], decision: Decision) -> Option<String> {
    decisions
        .iter()
        .find(|readiness| readiness.decision == decision)
        .and_then(|readiness| match readiness.state {
            DecisionState::Ready => None,
            DecisionState::Missing { question, .. } => Some(format!("ยังขาด: {question}")),
            DecisionState::Optional { unlock, .. } => Some(format!("เพิ่มได้: {unlock}")),
        })
}

/// Everything the plan still needs, with a route to each one.
#[component]
fn MissingInputs(id: i64, form: PlanForm) -> impl IntoView {
    let missing: Vec<_> = SECTION_TITLES
        .into_iter()
        .filter_map(|(slug, title)| {
            let readiness = form.section_readiness(slug);
            (readiness.tone == ReadinessTone::Missing).then_some((slug, title, readiness.label))
        })
        .collect();
    view! {
        <section class="card incomplete-state">
            <h2>"ยังคำนวณไม่ได้"</h2>
            <p>"หน้านี้จะไม่แสดงตัวเลขที่เดาเอา ยังขาดข้อมูลอยู่ตรงนี้"</p>
            <div class="section-list">
                {missing.into_iter().map(|(slug, title, status)| view! {
                    <A attr:class="section-card" href=format!("/plans/{id}/{slug}")>
                        <span><strong>{title}</strong><small>"ไปกรอกส่วนนี้"</small></span>
                        <span class="status warning">{status}</span>
                    </A>
                }).collect_view()}
            </div>
        </section>
    }
}

// ----------------------------------------------------------------- dashboard

#[component]
pub fn PlanDashboardView(record: PlanRecord) -> impl IntoView {
    let id = record.id;
    let name = record.form.name.clone();
    let year = record.season_year;
    let form = record.form.clone();
    let assets = record.asset_allocations.clone();
    let starting_capital = record.starting_capital;
    let decisions = form.decision_readiness(&assets, starting_capital);
    let analysis = record
        .form
        .to_plan()
        .ok()
        .map(|plan| calc::analyze_with_assets(&plan, &assets, starting_capital));

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">{year.map_or_else(|| "ฤดูกาล".into(), |year| format!("ฤดูกาล {year}"))}</p><h1>{name}</h1></div>
                <A attr:class="icon-button" href=format!("/plans?from={id}") attr:aria-label="เปิดรายการฤดูกาล">"×"</A>
            </header>
            {match analysis {
                Some(analysis) => view! { <DashboardFigures id=Some(id) analysis decisions/> }.into_any(),
                _ => view! { <MissingInputs id form/> <DecisionList plan_id=id decisions/> }.into_any(),
            }}
            <AnalysisLinks id/>
            <BottomNav plan_id=id active=NavSection::Home/>
        </section>
    }
}

/// Every figure the dashboard can show, each led by the owner's question
/// and followed by its accounting name. A figure whose inputs are absent
/// shows the missing question instead of a number, never a zero.
#[component]
pub(crate) fn DashboardFigures(
    /// `None` on the browser-only sample, which has no pages to link to.
    id: Option<i64>,
    analysis: Analysis,
    decisions: Vec<DecisionReadiness>,
) -> impl IntoView {
    let roi = roi_figure(&analysis);
    let payback = payback_figure(&analysis);
    let business = analysis.business;
    let revenue = analysis.revenue;
    let cost = analysis.cost;
    let health = analysis.health;
    let first_estimate = decisions
        .iter()
        .find(|readiness| readiness.decision == Decision::FirstEstimate)
        .map(|readiness| readiness.state)
        .unwrap_or(DecisionState::Ready);
    let profit_absent = absent_because(&decisions, Decision::FirstEstimate);
    let cash_absent = absent_because(&decisions, Decision::CashView);
    let market_absent = absent_because(&decisions, Decision::MarketComparison);
    let health_absent = absent_because(&decisions, Decision::HealthSelfReview);
    let revenue_absent = revenue
        .revenue
        .is_none()
        .then(|| profit_absent.clone().unwrap_or_else(|| ABSENT.to_owned()));

    view! {
        {match first_estimate {
            DecisionState::Ready => view! {
                <section class="card hero-card">
                    <span class="figure-label">
                        <span class="figure-words">"เหลือหลังหักค่าใช้จ่ายทั้งหมด"<small class="formal-term">"กำไรสุทธิ"</small></span>
                        <Explain explanation=explanations::NET_PROFIT label="เหลือหลังหักค่าใช้จ่ายทั้งหมด".into() plan_id=id/>
                    </span>
                    <strong class="hero-value">{baht(business.net_profit)}</strong>
                </section>
            }.into_any(),
            DecisionState::Missing { question, section } => view! {
                <section class="card hero-card incomplete-state">
                    <span class="figure-label"><span class="figure-words">"ปีนี้จะเหลือกำไรเท่าไร"<small class="formal-term">"กำไรสุทธิ"</small></span></span>
                    <h2>"ยังบอกไม่ได้"</h2>
                    <p>{format!("หน้านี้ไม่แสดงตัวเลขที่เดาเอา ยังขาด: {question}")}</p>
                    {id.map(|id| view! { <A attr:class="button primary" href=format!("/plans/{id}/{section}")>"ไปตอบข้อนี้"</A> })}
                </section>
            }.into_any(),
            DecisionState::Optional { .. } => ().into_any(),
        }}

        // With a figure at the top the six rows shrink to one line; while
        // the hero says "ยังบอกไม่ได้" the full list is the way forward.
        {id.map(|id| view! { <DecisionList plan_id=id decisions=decisions.clone() compact=matches!(first_estimate, DecisionState::Ready)/> })}

        <section class="figure-grid">
            <div class="card figure-tile">
                <span class="figure-label">
                    <span class="figure-words">"ต้นทุนต่อ 1 กก. ที่ขาย"<small class="formal-term">"ต้นทุนต่อกิโลกรัม"</small></span>
                    <Explain explanation=explanations::for_kpi(KpiKind::CostPerKg) label="ต้นทุนต่อ 1 กก. ที่ขาย".into() plan_id=id/>
                </span>
                {match (cost.cost_per_kg, &profit_absent) {
                    (Some(_), _) => view! { <strong>{with_unit(cost.cost_per_kg, "บาท/กก.")}</strong> }.into_any(),
                    (None, Some(reason)) => view! { <span class="figure-missing">{reason.clone()}</span> }.into_any(),
                    (None, None) => view! { <span class="figure-missing">{ABSENT}</span> }.into_any(),
                }}
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    <span class="figure-words">"ต้องขายอย่างน้อยกี่กิโลจึงไม่ขาดทุน"<small class="formal-term">"จุดคุ้มทุนด้านปริมาณ"</small></span>
                    <Explain explanation=explanations::BREAK_EVEN label="ต้องขายอย่างน้อยกี่กิโลจึงไม่ขาดทุน".into() plan_id=id/>
                </span>
                {match (business.break_even_kg, &profit_absent) {
                    (Some(_), _) => view! { <strong>{with_unit(business.break_even_kg, "กก.")}</strong> }.into_any(),
                    (None, Some(reason)) => view! { <span class="figure-missing">{reason.clone()}</span> }.into_any(),
                    (None, None) => view! { <span class="figure-missing">{ABSENT}</span> }.into_any(),
                }}
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    <span class="figure-words">"กำไรเทียบกับเงินก้อนที่ลงไป"<small class="formal-term">"ผลตอบแทนต่อเงินลงทุน (ROI)"</small></span>
                    <Explain explanation=explanations::ROI label="กำไรเทียบกับเงินก้อนที่ลงไป".into() plan_id=id/>
                </span>
                {match roi {
                    Ok(value) => view! { <strong>{value}</strong> }.into_any(),
                    Err(reason) => view! { <span class="figure-missing">{reason}</span> }.into_any(),
                }}
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    <span class="figure-words">"อีกกี่ปีเงินก้อนจะกลับมาครบ"<small class="formal-term">"ระยะคืนทุน"</small></span>
                    <Explain explanation=explanations::PAYBACK label="อีกกี่ปีเงินก้อนจะกลับมาครบ".into() plan_id=id/>
                </span>
                {match payback {
                    Ok(value) => view! { <strong>{value}</strong> }.into_any(),
                    Err(reason) => view! { <span class="figure-missing">{reason}</span> }.into_any(),
                }}
            </div>
        </section>

        <section class="card figure-list">
            <Figure label="ขายได้ทั้งฤดู" formal_term="รายได้รวม" value=baht(revenue.revenue) absent_reason=revenue_absent/>
            <Figure label="จ่ายทั้งฤดู รวมค่าเสื่อม" formal_term="ต้นทุนรวม" value=baht(cost.total_cost)
                explanation=explanations::VARIABLE_VERSUS_FIXED plan_id=id absent_reason=cost.total_cost.is_none().then(|| profit_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="เงินสดที่เหลือจากฤดูนี้" formal_term="กระแสเงินสดจากการดำเนินงาน" value=baht(business.operating_cash_flow)
                explanation=explanations::NET_PROFIT plan_id=id absent_reason=business.operating_cash_flow.is_none().then(|| cash_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="ราคาเฉลี่ยของทุกเกรดรวมกัน" formal_term="ราคาขายเฉลี่ยถ่วงน้ำหนัก" value=with_unit(revenue.weighted_price_per_kg, "บาท/กก.")
                explanation=explanations::WEIGHTED_PRICE plan_id=id absent_reason=revenue.weighted_price_per_kg.is_none().then(|| profit_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="เงินเหลือต่อ 1 กก. ก่อนจ่ายค่าใช้จ่ายประจำ" formal_term="ส่วนเกินต่อหน่วย" value=with_unit(business.contribution_per_kg, "บาท/กก.")
                explanation=explanations::CONTRIBUTION_PER_KG plan_id=id absent_reason=business.contribution_per_kg.is_none().then(|| profit_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="ขายได้เกินขั้นต่ำที่ไม่ขาดทุนกี่กิโล" formal_term="ส่วนเผื่อความปลอดภัย" value=with_unit(business.safety_margin_kg, "กก.")
                explanation=explanations::SAFETY_MARGIN plan_id=id absent_reason=business.safety_margin_kg.is_none().then(|| profit_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="ของที่มีเทียบกับที่ผู้ซื้อคุยไว้" formal_term="การตอบสนองตลาด" value=percent(revenue.market_fulfillment)
                explanation=explanations::MARKET_FULFILLMENT plan_id=id absent_reason=revenue.market_fulfillment.is_none().then(|| market_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
            <Figure label="คะแนนที่ประเมินสวนเอง" formal_term="คะแนนสุขภาพธุรกิจ" value=score(health.overall_score)
                explanation=explanations::HEALTH_SCORE plan_id=id absent_reason=health.overall_score.is_none().then(|| health_absent.clone().unwrap_or_else(|| ABSENT.to_owned()))/>
        </section>
    }
}

#[component]
fn AnalysisLinks(id: i64) -> impl IntoView {
    view! {
        <section class="section-list">
            <A attr:class="section-card" href=format!("/plans/{id}/analysis")>
                <span><strong>"วิเคราะห์"</strong><small>"ประสิทธิภาพ ตรวจสอบ ภาษี สถานการณ์"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
            <A attr:class="section-card" href=format!("/plans/{id}")>
                <span><strong>"กรอกข้อมูล"</strong><small>"ห้าส่วนหลักของแผนฤดูกาลนี้"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
        </section>
    }
}

// ------------------------------------------------------------------ analysis

#[component]
pub fn PlanAnalysisView(record: PlanRecord) -> impl IntoView {
    let id = record.id;
    let name = record.form.name.clone();
    let year = record.season_year;
    let form = record.form.clone();
    let assets = record.asset_allocations.clone();
    let starting_capital = record.starting_capital;
    let decisions = form.decision_readiness(&assets, starting_capital);
    let analysis = record
        .form
        .to_plan()
        .ok()
        .map(|plan| calc::analyze_with_assets(&plan, &assets, starting_capital));
    let tab = RwSignal::new("efficiency");

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">{year.map_or_else(|| "วิเคราะห์".into(), |year| format!("วิเคราะห์ · ฤดูกาล {year}"))}</p><h1>{name}</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}/dashboard") attr:aria-label="กลับไปหน้าแรกของแผน">"×"</A>
            </header>
            <DecisionList plan_id=id decisions compact=true/>
            {match analysis {
                Some(analysis) => view! { <AnalysisTabs id analysis tab/> }.into_any(),
                None => view! { <MissingInputs id form/> }.into_any(),
            }}
            <BottomNav plan_id=id active=NavSection::Analysis/>
        </section>
    }
}

#[component]
fn AnalysisTabs(id: i64, analysis: Analysis, tab: RwSignal<&'static str>) -> impl IntoView {
    let tabs = [
        ("efficiency", "ประสิทธิภาพ"),
        ("checks", "ตรวจสอบ"),
        ("tax", "ภาษี"),
        ("scenario", "สถานการณ์"),
    ];
    let efficiency = analysis.efficiency;
    let checks = analysis.checks;
    let tax = analysis.tax;
    let scenario = analysis.scenario;
    let health = analysis.health;

    view! {
        <nav class="segmented" role="tablist" aria-label="มุมมองการวิเคราะห์">
            {tabs.into_iter().map(|(key, title)| view! {
                <button
                    type="button"
                    role="tab"
                    class=move || if tab.get() == key { "segment selected" } else { "segment" }
                    aria-selected=move || (tab.get() == key).to_string()
                    on:click=move |_| tab.set(key)
                >{title}</button>
            }).collect_view()}
        </nav>

        // Every panel is rendered and the inactive ones carry `hidden`, so the
        // server sends the whole analysis in one response and a reader who
        // never runs the script still receives it.
        <div role="tabpanel" aria-label="ประสิทธิภาพ" hidden=move || tab.get() != "efficiency">
            <EfficiencyPanel id kpis=efficiency health/>
        </div>
        <div role="tabpanel" aria-label="ตรวจสอบ" hidden=move || tab.get() != "checks">
            <ChecksPanel id checks/>
        </div>
        <div role="tabpanel" aria-label="ภาษี" hidden=move || tab.get() != "tax">
            <TaxPanel id tax/>
        </div>
        <div role="tabpanel" aria-label="สถานการณ์" hidden=move || tab.get() != "scenario">
            <ScenarioPanel scenario/>
        </div>
    }
}

#[component]
fn EfficiencyPanel(id: i64, kpis: Vec<KpiResult>, health: calc::HealthAnalysis) -> impl IntoView {
    view! {
        <section class="card kpi-list">
            <h2>"ได้ผลผลิตเท่าไรต่อสิ่งที่ใส่ลงไป"</h2><small class="formal-term">"ประสิทธิภาพ · ตัวชี้วัดผลงาน (KPI)"</small>
            <p class="caption">"แสดงเฉพาะตัวที่มีปริมาณและหน่วยให้หาร ตัวที่ไม่ได้กรอกไม่ถูกนับว่าขาด เป้าเป็นของคุณเอง ระบบไม่ตัดสินว่าตัวเลขไหนดี"</p>
            {kpis.into_iter().filter(|kpi| kpi.actual.is_some()).map(|kpi| view! { <KpiRow id kpi/> }).collect_view()}
        </section>
        <section class="card figure-list">
            <div class="section-title">
                <div><h2>"สวนพร้อมแค่ไหน ตามที่ประเมินเอง"</h2><small class="formal-term">"คะแนนสุขภาพธุรกิจ"</small></div>
                {health.overall_score.map(|_| view! { <strong>{score(health.overall_score)}</strong> })}
            </div>
            <p class="caption">"คะแนนมาจากคำตอบ 12 ข้อที่คุณให้เอง เป็นภาพที่คุณเห็นสวนของตัวเอง ไม่ใช่การวินิจฉัย"</p>
            {if health.overall_score.is_none() {
                // No answers yet: one route to the questions, not six empty rows.
                view! { <A attr:class="text-button" href=format!("/plans/{id}/health")>"เพิ่มได้: แบบประเมินสวน 12 ข้อ"</A> }.into_any()
            } else {
                view! {
                    {health.overall_status.map(|status| {
                        let (label, class) = health_status_label(status);
                        view! { <span class=class>{label}</span> }
                    })}
                    {health.dimensions.into_iter().map(|dimension| {
                        let label = dimension_label(dimension.dimension);
                        let status = dimension.status.map(health_status_label);
                        view! {
                            <div class="figure-row">
                                <span class="figure-label">{label}</span>
                                {match dimension.average {
                                    Some(_) => view! {
                                        <span class="figure-value">
                                            <strong>{score(dimension.average)}</strong>
                                            {status.map(|(text, class)| view! { <span class=class>{text}</span> })}
                                        </span>
                                    }.into_any(),
                                    None => view! { <span class="figure-value figure-missing">"ยังตอบไม่ครบ 2 ข้อ"</span> }.into_any(),
                                }}
                            </div>
                        }
                    }).collect_view()}
                }.into_any()
            }}
    </section>
    }
}

#[component]
fn KpiRow(id: i64, kpi: KpiResult) -> impl IntoView {
    let (label, unit) = kpi_label(kpi.kind);
    let share = kpi_is_share(kpi.kind);
    let actual = if share {
        percent(kpi.actual)
    } else {
        with_unit(kpi.actual, unit)
    };
    let verdict = kpi.verdict;
    let target = kpi.target;

    view! {
        <div class="kpi-row">
            <span class="figure-label">
                {label}
                <Explain explanation=explanations::for_kpi(kpi.kind) label=label.to_owned() plan_id=Some(id)/>
            </span>
            <span class="kpi-actual">{actual}</span>
            {match (target, verdict) {
                (Some(target), Some(verdict)) => {
                    let target = if share { percent(Some(target)) } else { with_unit(Some(target), unit) };
                    let (word, class) = match verdict {
                        KpiVerdict::Met => ("ถึงเป้าที่ตั้งไว้", "status good"),
                        KpiVerdict::Improve => ("ต่ำกว่าเป้าที่ตั้งไว้", "status warning"),
                    };
                    view! {
                        <span class="kpi-target">"เป้า "{target}</span>
                        <span class=class>{word}</span>
                    }.into_any()
                }
                _ => view! {
                    <A attr:class="kpi-no-target" href=format!("/plans/{id}/targets")>"ตั้งเป้าในขั้นสูง"</A>
                }.into_any(),
            }}
        </div>
    }
}

#[component]
fn ChecksPanel(id: i64, checks: calc::CompletenessAnalysis) -> impl IntoView {
    let (readiness, readiness_class) = match checks.overall {
        Readiness::Ready => ("ไม่พบข้อที่ควรตรวจ", "status good"),
        Readiness::NeedsReview => ("มีข้อที่ควรตรวจ", "status muted"),
    };
    view! {
        <section class="card">
            <div class="section-title">
                <div><h2>"ข้อที่ควรตรวจก่อนเชื่อตัวเลข"</h2><small class="formal-term">"ตรวจสอบความครบถ้วน"</small></div>
                <span class=readiness_class>{readiness}</span>
            </div>
            <p class="caption">"แต่ละข้อเป็นการเทียบตัวเลขที่กรอกไว้ ไม่ใช่คำตัดสินเรื่องสวน กดเพื่อไปดูข้อมูลที่เกี่ยวข้อง"</p>
            <div class="section-list">
                {checks.checks.into_iter().map(|check| view! { <CheckRow id check/> }).collect_view()}
            </div>
        </section>
    }
}

#[component]
fn CheckRow(id: i64, check: CheckResult) -> impl IntoView {
    let (label, formal, section) = check_label(check.kind);
    let (status, class) = check_status_label(check.status);
    view! {
        <A attr:class="section-card" href=format!("/plans/{id}/{section}")>
            <span><strong>{label}</strong><small class="formal-term">{formal}</small></span>
            <span class=class>{status}</span>
        </A>
    }
}

#[component]
fn TaxPanel(id: i64, tax: calc::TaxAnalysis) -> impl IntoView {
    let actual = tax.actual_expense.clone();
    let flat = tax.flat_sixty_percent.clone();
    let entered = tax.deduction_count > 0;
    let state_line = if entered {
        format!(
            "หักลดหย่อนแล้ว {} รายการ รวม {} บาท",
            tax.deduction_count,
            baht_amount(tax.deduction_total)
        )
    } else {
        "ยังไม่ได้หักลดหย่อน ตัวเลขนี้จึงสูงกว่าภาษีจริง".to_owned()
    };
    let link_label = if entered {
        "ดูหรือแก้"
    } else {
        "กรอกลดหย่อน"
    };

    view! {
        <section class="card">
            <div class="section-title">
                <div><h2>"ถ้าต้องเสียภาษี น่าจะประมาณเท่าไร"</h2><small class="formal-term">"ภาษีเงินได้บุคคลธรรมดา · ประมาณการ"</small></div>
                <Explain explanation=explanations::TAX label="ถ้าต้องเสียภาษี น่าจะประมาณเท่าไร".into() plan_id=Some(id)/>
            </div>
            <p class="caption tax-disclaimer">{explanations::TAX_DISCLAIMER}</p>
            <p class="tax-deduction-state" class:tax-deduction-missing=!entered>
                <span>{state_line}</span>
                <A attr:class="button secondary compact" href=format!("/plans/{id}/tax-deductions")>{link_label}</A>
            </p>
            <div class="tax-methods">
                <TaxMethod
                    title="หักค่าใช้จ่ายตามจริง"
                    method=actual
                />
                <TaxMethod
                    title="หักค่าใช้จ่ายแบบเหมา"
                    method=flat
                />
            </div>
        </section>
    }
}

#[component]
fn TaxMethod(title: &'static str, method: TaxMethodAnalysis) -> impl IntoView {
    let exceeded = method.deductions_exceed_income;
    view! {
        <div class="card tax-method">
            <div class="section-title"><h3>{title}</h3></div>
            <Figure label="ขายได้ทั้งฤดู" formal_term="รายได้" value=baht(method.income)/>
            <Figure label="หักค่าใช้จ่ายออก" formal_term="ค่าใช้จ่ายที่หักได้" value=baht(method.expense)/>
            <Figure label="หักลดหย่อนอีก" formal_term="ค่าลดหย่อนรวม" value=baht(Some(method.deductions))/>
            <Figure label="เหลือที่ต้องคิดภาษี" formal_term="เงินได้สุทธิ" value=baht(method.taxable_income)/>
            <Figure label="ภาษีโดยประมาณ" value=baht(method.estimated_tax)/>
            {exceeded.then(|| view! { <p class="caption">"ลดหย่อนมากกว่าเงินได้หลังหักค่าใช้จ่าย จึงไม่มีภาษี"</p> })}
            <Figure label="คิดเป็นกี่เปอร์เซ็นต์ของที่ขายได้" formal_term="อัตราภาษีเฉลี่ย" value=percent(method.average_tax_rate)/>
        </div>
    }
}

#[component]
fn ScenarioPanel(scenario: calc::ScenarioAnalysis) -> impl IntoView {
    let yield_index = RwSignal::new(2_usize);
    let price_index = RwSignal::new(2_usize);
    let profits = scenario.profits;
    let yield_changes = scenario.yield_changes;
    let price_changes = scenario.price_changes;

    let selected = move || profits[yield_index.get()][price_index.get()];
    let yield_label = move || percent(Some(yield_changes[yield_index.get()]));
    let price_label = move || percent(Some(price_changes[price_index.get()]));

    view! {
        <section class="card scenario-card">
            <h2>"ถ้าราคาหรือผลผลิตเปลี่ยนไป จะเหลือเท่าไร"</h2><small class="formal-term">"การวิเคราะห์สถานการณ์ · กำไรสุทธิ"</small>
            <p class="caption">"เลื่อนดูว่าที่เหลือหลังหักค่าใช้จ่ายทั้งหมดเปลี่ยนไปเท่าไร ทั้งสองแกนใช้ตัวเลขชุดเดียวกับตารางเต็ม"</p>
            <strong class="hero-value">{move || baht(selected())}</strong>

            <label class="slider-field">
                <span>"ผลผลิตเปลี่ยน "{yield_label}</span>
                <input
                    type="range" min="0" max="4" step="1"
                    prop:value=move || yield_index.get().to_string()
                    on:input=move |event| {
                        if let Ok(index) = event_target_value(&event).parse::<usize>() {
                            yield_index.set(index.min(4));
                        }
                    }
                />
            </label>
            <label class="slider-field">
                <span>"ราคาเปลี่ยน "{price_label}</span>
                <input
                    type="range" min="0" max="4" step="1"
                    prop:value=move || price_index.get().to_string()
                    on:input=move |event| {
                        if let Ok(index) = event_target_value(&event).parse::<usize>() {
                            price_index.set(index.min(4));
                        }
                    }
                />
            </label>

            <details class="scenario-table">
                <summary>"ดูตารางทั้งหมด"</summary>
                <div class="table-scroll">
                    <table>
                        <caption>"กำไรสุทธิ บาท ตามผลผลิตและราคาที่เปลี่ยนไป"</caption>
                        <thead>
                            <tr>
                                <th scope="col">"ผลผลิต ลง · ราคา ขวา"</th>
                                {price_changes.into_iter().map(|change| view! {
                                    <th scope="col">{percent(Some(change))}</th>
                                }).collect_view()}
                            </tr>
                        </thead>
                        <tbody>
                            {(0..5).map(|row| view! {
                                <tr>
                                    <th scope="row">{percent(Some(yield_changes[row]))}</th>
                                    {(0..5).map(|column| view! {
                                        <td>{money_or_absent(profits[row][column])}</td>
                                    }).collect_view()}
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </details>
        </section>
    }
}

fn money_or_absent(value: Option<Decimal>) -> String {
    value.map_or_else(|| ABSENT.into(), baht_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn an_absent_figure_is_named_rather_than_shown_as_zero() {
        assert_eq!(baht(None), ABSENT);
        assert_eq!(with_unit(None, "กก."), ABSENT);
        assert_eq!(percent(None), ABSENT);
        assert_eq!(score(None), ABSENT);
    }

    #[test]
    fn an_amount_is_whole_baht_with_thousands_separators() {
        assert_eq!(baht(Some(Decimal::new(83_460_012, 2))), "834,600 บาท");
        assert_eq!(baht(Some(Decimal::new(83_460_050, 2))), "834,601 บาท");
    }

    #[test]
    fn a_ratio_is_shown_as_a_percentage() {
        assert_eq!(percent(Some(Decimal::new(798, 3))), "79.80%");
    }

    #[test]
    fn a_health_average_is_shown_out_of_five() {
        assert_eq!(score(Some(Decimal::new(30, 1))), "3.00/5");
    }

    #[test]
    fn every_kpi_has_a_label_and_a_unit() {
        for kind in KpiKind::ALL {
            let (label, unit) = kpi_label(kind);
            assert!(!label.is_empty() && !unit.is_empty());
        }
    }

    #[test]
    fn every_check_routes_to_an_input_section_that_exists() {
        for kind in [
            CheckKind::GradeSharesTotalOne,
            CheckKind::SellableYieldPositive,
            CheckKind::PriceAboveVariableCost,
            CheckKind::InvestmentPositive,
            CheckKind::TotalCostLinked,
            CheckKind::HealthAnswersComplete,
        ] {
            let (_, _, section) = check_label(kind);
            assert!(
                SECTION_TITLES.iter().any(|(slug, _)| *slug == section),
                "{kind:?} routes to {section}, which is not an input section"
            );
        }
    }
}
