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
    plan_form::PlanForm,
    plan_ui::{BottomNav, money},
    plans::PlanRecord,
};

/// Shown wherever a figure cannot be computed from what the owner has entered.
const ABSENT: &str = "ยังไม่มีข้อมูล";
const NO_TARGET: &str = "ยังไม่ได้ตั้งเป้า";

const SECTION_TITLES: [(&str, &str); 6] = [
    ("market", "ตลาด"),
    ("production", "ผลผลิตและเกรด"),
    ("variable-costs", "ต้นทุนผันแปร"),
    ("fixed-costs", "ต้นทุนคงที่"),
    ("targets", "เป้าหมาย"),
    ("health", "สุขภาพสวน"),
];

// ---------------------------------------------------------------- formatting

fn baht(value: Option<Decimal>) -> String {
    value.map_or_else(|| ABSENT.into(), |value| format!("{} บาท", money(value)))
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
const fn check_label(kind: CheckKind) -> (&'static str, &'static str) {
    match kind {
        CheckKind::GradeSharesTotalOne => ("สัดส่วนเกรดรวมได้ 100%", "production"),
        CheckKind::SellableYieldPositive => ("มีผลผลิตที่ขายได้", "production"),
        CheckKind::PriceAboveVariableCost => ("ราคาขายสูงกว่าต้นทุนผันแปร", "variable-costs"),
        CheckKind::InvestmentPositive => ("มีฐานเงินลงทุน", "fixed-costs"),
        CheckKind::TotalCostLinked => ("ต้นทุนรวมตรงกับรายการที่กรอก", "variable-costs"),
        CheckKind::HealthAnswersComplete => ("ตอบคำถามสุขภาพครบ 12 ข้อ", "health"),
    }
}

const fn check_status_label(status: CheckStatus) -> (&'static str, &'static str) {
    match status {
        CheckStatus::Ok => ("ผ่าน", "status good"),
        CheckStatus::Warning => ("ต้องแก้", "status bad-text"),
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
        HealthStatus::Strong => ("แข็งแรง", "status good"),
        HealthStatus::Watch => ("เฝ้าระวัง", "status muted"),
        HealthStatus::ImproveUrgently => ("ต้องเร่งปรับปรุง", "status bad-text"),
    }
}

// ---------------------------------------------------------------- components

/// The `ⓘ` affordance. Four questions, always in the same order.
#[component]
fn Explain(explanation: Explanation, label: String) -> impl IntoView {
    view! {
        <details class="figure-explanation">
            <summary aria-label=format!("อธิบาย{label}")>"ⓘ"</summary>
            <div>
                <h3>"คืออะไร"</h3><p>{explanation.what}</p>
                <h3>"ใช้ยังไง"</h3><p>{explanation.how}</p>
                <h3>"ทำไมต้องมี"</h3><p>{explanation.why}</p>
                <h3>"ไม่ใส่ได้ไหม"</h3><p>{explanation.missing}</p>
            </div>
        </details>
    }
}

#[component]
fn Figure(
    label: &'static str,
    value: String,
    #[prop(optional)] explanation: Option<Explanation>,
) -> impl IntoView {
    view! {
        <div class="figure-row">
            <span class="figure-label">
                {label}
                {explanation.map(|explanation| view! {
                    <Explain explanation label=label.to_owned()/>
                })}
            </span>
            <strong class="figure-value">{value}</strong>
        </div>
    }
}

/// Everything the plan still needs, with a route to each one.
#[component]
fn MissingInputs(id: i64, form: PlanForm) -> impl IntoView {
    let missing: Vec<_> = SECTION_TITLES
        .into_iter()
        .filter(|(slug, _)| !form.section_complete(slug))
        .collect();
    view! {
        <section class="card incomplete-state">
            <h2>"ยังคำนวณไม่ได้"</h2>
            <p>"หน้านี้จะไม่แสดงตัวเลขที่เดาเอา ยังขาดข้อมูลอยู่ตรงนี้"</p>
            <div class="section-list">
                {missing.into_iter().map(|(slug, title)| view! {
                    <A attr:class="section-card" href=format!("/plans/{id}/{slug}")>
                        <span><strong>{title}</strong><small>"ไปกรอกส่วนนี้"</small></span>
                        <span class="status muted">"ยังไม่ครบ"</span>
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
    let form = record.form.clone();
    let analysis = record.form.to_plan().ok().map(|plan| calc::analyze(&plan));

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">"หน้าแรก"</p><h1>{name}</h1></div>
                <A attr:class="icon-button" href="/plans" attr:aria-label="กลับไปรายการแผน">"×"</A>
            </header>
            {match analysis {
                Some(analysis) if analysis.business.net_profit.is_some() =>
                    view! { <DashboardFigures analysis/> }.into_any(),
                _ => view! { <MissingInputs id form/> }.into_any(),
            }}
            <AnalysisLinks id/>
            <BottomNav plan_id=Some(id)/>
        </section>
    }
}

#[component]
fn DashboardFigures(analysis: Analysis) -> impl IntoView {
    let business = analysis.business;
    let revenue = analysis.revenue;
    let cost = analysis.cost;
    let health = analysis.health;

    view! {
        <section class="card hero-card">
            <span class="figure-label">
                "กำไรสุทธิ"
                <Explain explanation=explanations::NET_PROFIT label="กำไรสุทธิ".into()/>
            </span>
            <strong class="hero-value">{baht(business.net_profit)}</strong>
        </section>

        <section class="figure-grid">
            <div class="card figure-tile">
                <span class="figure-label">
                    "ต้นทุนต่อกิโลกรัม"
                    <Explain explanation=explanations::for_kpi(KpiKind::CostPerKg) label="ต้นทุนต่อกิโลกรัม".into()/>
                </span>
                <strong>{with_unit(cost.cost_per_kg, "บาท/กก.")}</strong>
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    "จุดคุ้มทุน"
                    <Explain explanation=explanations::BREAK_EVEN label="จุดคุ้มทุน".into()/>
                </span>
                <strong>{with_unit(business.break_even_kg, "กก.")}</strong>
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    "ROI"
                    <Explain explanation=explanations::ROI label="ROI".into()/>
                </span>
                <strong>{percent(business.roi)}</strong>
            </div>
            <div class="card figure-tile">
                <span class="figure-label">
                    "ระยะคืนทุน"
                    <Explain explanation=explanations::PAYBACK label="ระยะคืนทุน".into()/>
                </span>
                <strong>{with_unit(business.payback_years, "ปี")}</strong>
            </div>
        </section>

        <section class="card figure-list">
            <Figure label="รายได้รวม" value=baht(revenue.revenue)/>
            <Figure label="ต้นทุนรวม" value=baht(cost.total_cost)
                explanation=explanations::VARIABLE_VERSUS_FIXED/>
            <Figure label="กระแสเงินสด" value=baht(business.operating_cash_flow)
                explanation=explanations::NET_PROFIT/>
            <Figure label="ราคาขายเฉลี่ยถ่วงน้ำหนัก" value=with_unit(revenue.weighted_price_per_kg, "บาท/กก.")
                explanation=explanations::WEIGHTED_PRICE/>
            <Figure label="ส่วนเกินต่อหน่วย" value=with_unit(business.contribution_per_kg, "บาท/กก.")
                explanation=explanations::CONTRIBUTION_PER_KG/>
            <Figure label="ส่วนเผื่อความปลอดภัย" value=with_unit(business.safety_margin_kg, "กก.")
                explanation=explanations::SAFETY_MARGIN/>
            <Figure label="ตอบสนองตลาด" value=percent(revenue.market_fulfillment)
                explanation=explanations::MARKET_FULFILLMENT/>
            <Figure label="คะแนนสุขภาพธุรกิจ" value=score(health.overall_score)
                explanation=explanations::HEALTH_SCORE/>
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
                <span><strong>"กรอกข้อมูล"</strong><small>"หกส่วนของแผนฤดูกาลนี้"</small></span>
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
    let form = record.form.clone();
    let analysis = record.form.to_plan().ok().map(|plan| calc::analyze(&plan));
    let tab = RwSignal::new("efficiency");

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">"วิเคราะห์"</p><h1>{name}</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}/dashboard") attr:aria-label="กลับไปหน้าแรกของแผน">"×"</A>
            </header>
            {match analysis {
                Some(analysis) => view! { <AnalysisTabs id analysis tab/> }.into_any(),
                None => view! { <MissingInputs id form/> }.into_any(),
            }}
            <BottomNav plan_id=Some(id)/>
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
            <TaxPanel tax/>
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
            <h2>"ประสิทธิภาพ"</h2>
            {kpis.into_iter().map(|kpi| view! { <KpiRow id kpi/> }).collect_view()}
        </section>
        <section class="card figure-list">
            <div class="section-title">
                <h2>"สุขภาพธุรกิจ"</h2>
                <strong>{score(health.overall_score)}</strong>
            </div>
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
                        <span class="figure-value">
                            <strong>{score(dimension.average)}</strong>
                            {status.map(|(text, class)| view! { <span class=class>{text}</span> })}
                        </span>
                    </div>
                }
            }).collect_view()}
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
                <Explain explanation=explanations::for_kpi(kpi.kind) label=label.to_owned()/>
            </span>
            <span class="kpi-actual">{actual}</span>
            {match (target, verdict) {
                (Some(target), Some(verdict)) => {
                    let target = if share { percent(Some(target)) } else { with_unit(Some(target), unit) };
                    let (word, class) = match verdict {
                        KpiVerdict::Met => ("ถึงเป้า", "status good"),
                        KpiVerdict::Improve => ("ต้องปรับปรุง", "status bad-text"),
                    };
                    view! {
                        <span class="kpi-target">"เป้า "{target}</span>
                        <span class=class>{word}</span>
                    }.into_any()
                }
                _ => view! {
                    <A attr:class="kpi-no-target" href=format!("/plans/{id}/targets")>{NO_TARGET}</A>
                }.into_any(),
            }}
        </div>
    }
}

#[component]
fn ChecksPanel(id: i64, checks: calc::CompletenessAnalysis) -> impl IntoView {
    let (readiness, readiness_class) = match checks.overall {
        Readiness::Ready => ("พร้อมใช้ตัดสินใจ", "status good"),
        Readiness::NeedsReview => ("ยังต้องตรวจ", "status muted"),
    };
    view! {
        <section class="card">
            <div class="section-title">
                <h2>"ตรวจสอบความครบถ้วน"</h2>
                <span class=readiness_class>{readiness}</span>
            </div>
            <div class="section-list">
                {checks.checks.into_iter().map(|check| view! { <CheckRow id check/> }).collect_view()}
            </div>
        </section>
    }
}

#[component]
fn CheckRow(id: i64, check: CheckResult) -> impl IntoView {
    let (label, section) = check_label(check.kind);
    let (status, class) = check_status_label(check.status);
    view! {
        <A attr:class="section-card" href=format!("/plans/{id}/{section}")>
            <span><strong>{label}</strong><small>"ไปที่ข้อมูลที่เกี่ยวข้อง"</small></span>
            <span class=class>{status}</span>
        </A>
    }
}

#[component]
fn TaxPanel(tax: calc::TaxAnalysis) -> impl IntoView {
    let actual = tax.actual_expense.clone();
    let flat = tax.flat_sixty_percent.clone();
    let cheaper = match (actual.estimated_tax, flat.estimated_tax) {
        (Some(actual), Some(flat)) if actual < flat => Some("actual"),
        (Some(actual), Some(flat)) if flat < actual => Some("flat"),
        _ => None,
    };

    view! {
        <section class="card">
            <div class="section-title">
                <h2>"ภาษีเงินได้ ประมาณการ"</h2>
                <Explain explanation=explanations::TAX label="ภาษีสองวิธี".into()/>
            </div>
            <div class="tax-methods">
                <TaxMethod
                    title="หักค่าใช้จ่ายตามจริง"
                    method=actual
                    cheaper=cheaper == Some("actual")
                />
                <TaxMethod
                    title="หักค่าใช้จ่ายแบบเหมา"
                    method=flat
                    cheaper=cheaper == Some("flat")
                />
            </div>
            <p class="caption tax-disclaimer">{explanations::TAX_DISCLAIMER}</p>
        </section>
    }
}

#[component]
fn TaxMethod(title: &'static str, method: TaxMethodAnalysis, cheaper: bool) -> impl IntoView {
    view! {
        <div class=if cheaper { "card tax-method cheaper" } else { "card tax-method" }>
            <div class="section-title">
                <h3>{title}</h3>
                <Show when=move || cheaper>
                    <span class="status good">"เสียน้อยกว่า"</span>
                </Show>
            </div>
            <Figure label="รายได้" value=baht(method.income)/>
            <Figure label="หักค่าใช้จ่าย" value=baht(method.expense)/>
            <Figure label="ค่าลดหย่อนส่วนตัว" value=baht(Some(method.personal_allowance))/>
            <Figure label="เงินได้สุทธิ" value=baht(method.taxable_income)/>
            <Figure label="ภาษีโดยประมาณ" value=baht(method.estimated_tax)/>
            <Figure label="อัตราภาษีเฉลี่ย" value=percent(method.average_tax_rate)/>
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
            <h2>"ถ้าราคาหรือผลผลิตเปลี่ยนไป"</h2>
            <p class="caption">"เลื่อนดูว่ากำไรสุทธิเปลี่ยนไปเท่าไร ทั้งสองแกนใช้ตัวเลขชุดเดียวกับตารางเต็ม"</p>
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
    value.map_or_else(|| ABSENT.into(), money)
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
    fn money_carries_two_decimals_and_thousands_separators() {
        assert_eq!(baht(Some(Decimal::new(83_460_012, 2))), "834,600.12 บาท");
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
            let (_, section) = check_label(kind);
            assert!(
                SECTION_TITLES.iter().any(|(slug, _)| *slug == section),
                "{kind:?} routes to {section}, which is not an input section"
            );
        }
    }
}
