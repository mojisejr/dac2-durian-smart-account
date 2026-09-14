use calc::{
    CashKind, ComparisonMetric, CostSectionState, HealthQuestion, PriceSource, VariableCostKind,
    YieldSource,
};
use leptos::{form::ActionForm, prelude::*};
use leptos_router::{
    components::A,
    hooks::{use_params_map, use_query_map},
};

use crate::{
    analysis_ui::{PlanAnalysisView, PlanDashboardView},
    auth::{Logout, current_user_email},
    plan_form::{
        DecisionReadiness, DecisionState, ExpenseClassification, FixedCostForm, GradeEntry,
        GradeForm, PlanForm, ReadinessTone, UnclassifiedExpenseForm, VariableCostForm,
    },
    plans::{
        CreateSeason, FinalizeActual, PlanRecord, SaveActualDraft, SavePlan, SaveQuickStep,
        SeasonHistoryItem, SwitchForecastMode, UpdateSeasonMetadata, list_plans,
        load_season_history, quick_resume_path,
    },
};

const MAIN_SECTIONS: [(&str, &str, &str); 6] = [
    ("market", "ขายให้ใครและขายทางไหน", "ข้อมูลตลาด"),
    (
        "production",
        "คาดว่าจะขายได้เท่าไร ราคาเท่าไร",
        "ผลผลิตขายได้และราคาขาย",
    ),
    (
        "expenses",
        "จำค่าใช้จ่ายได้แต่ยังไม่รู้ว่าเป็นแบบไหน",
        "จดไว้ก่อน แล้วค่อยบอกประเภท",
    ),
    ("variable-costs", "ค่าใช้จ่ายที่เพิ่มเมื่อทำหรือขายมากขึ้น", "ต้นทุนผันแปร"),
    ("fixed-costs", "แม้ปีนี้ไม่มีทุเรียนขาย ยังต้องจ่ายอะไรอยู่", "ต้นทุนคงที่"),
    ("health", "ทบทวนความพร้อมของสวน", "สุขภาพสวน"),
];

const EDITABLE_SECTIONS: [(&str, &str, &str); 7] = [
    ("market", "ตลาด", "ผู้ซื้อ ยอดที่คุยไว้ และช่องทางขาย"),
    ("production", "ผลผลิตและราคา", "กิโลที่คาดว่าจะขายได้ และราคาขาย"),
    ("expenses", "ค่าใช้จ่ายที่จำได้", "จดไว้ก่อน แล้วค่อยบอกว่าเป็นแบบไหน"),
    ("variable-costs", "ต้นทุนผันแปร", "ค่าใช้จ่ายที่เพิ่มเมื่อทำหรือขายมากขึ้น"),
    ("fixed-costs", "ต้นทุนคงที่", "ค่าใช้จ่ายที่ยังมีแม้ไม่มีทุเรียนขาย"),
    ("health", "สุขภาพสวน", "12 คำถาม 6 มิติ"),
    (
        "targets",
        "เป้าหมาย KPI",
        "ตัวเลขเปรียบเทียบขั้นสูงที่เจ้าของกำหนดเอง",
    ),
];

#[component]
pub fn PlansPage() -> impl IntoView {
    let logout = ServerAction::<Logout>::new();
    let user = Resource::new(|| (), |_| current_user_email());
    let plans = Resource::new(|| (), |_| list_plans());
    let query = use_query_map();
    let return_to = move || {
        query.with(|params| {
            params
                .get("from")
                .and_then(|value| value.parse::<i64>().ok())
        })
    };

    view! {
        <section class="page-stack">
            <header class="page-heading">
                <div>
                    <p class="eyebrow">"พื้นที่ส่วนตัว"</p>
                    <h1>"ฤดูกาลของฉัน"</h1>
                </div>
                <Suspense fallback=move || view! { <span>"…"</span> }>
                    {move || user.get().map(|result| view! {
                        <span class="user-email">{result.ok().flatten().unwrap_or_default()}</span>
                    })}
                </Suspense>
            </header>

            <Show when=move || return_to().is_some()>
                <A
                    attr:class="button secondary season-back"
                    href=move || format!("/plans/{}", return_to().unwrap_or_default())
                >"‹ กลับไปฤดูกาลที่เปิดอยู่"</A>
            </Show>

            <Suspense fallback=move || view! { <p>"กำลังอ่านฤดูกาล…"</p> }>
                {move || plans.get().map(|result| match result {
                    Ok(items) if items.is_empty() => view! {
                        <section class="card starter-card first-season">
                            <p class="eyebrow">"เริ่มต้น"</p>
                            <h2>"ลองดูก่อน หรือสร้างฤดูกาลแรก"</h2>
                            <p>"ข้อมูลตัวอย่างให้ลองเปลี่ยนตัวเลขได้โดยไม่บันทึกลงประวัติ"</p>
                            <div class="actions">
                                <A attr:class="button primary" href="/demo">"ดูตัวอย่างการใช้งาน"</A>
                                <A attr:class="button secondary" href="/plans/new">"สร้างฤดูกาลแรก"</A>
                            </div>
                        </section>
                    }.into_any(),
                    Ok(items) => {
                        let latest_year = items.iter().filter_map(|plan| plan.season_year).max();
                        view! {
                            <section class="card starter-card compact-starter">
                                <h2>"เริ่มฤดูกาลใหม่"</h2>
                                <p>"กำหนดปี ชื่อ และบันทึกก่อนสร้าง ฤดูกาลหนึ่งรวมข้อมูลจากทุกแปลง"</p>
                                <div class="actions">
                                    <A attr:class="button primary" href="/plans/new">"เริ่มฤดูกาลใหม่"</A>
                                    <A attr:class="button secondary" href="/history">"ดูประวัติผลจริง"</A>
                                    <A attr:class="button text-button" href="/demo">"ดูตัวอย่างการใช้งาน"</A>
                                </div>
                            </section>
                            <section class="plan-list" aria-label="รายการฤดูกาล">
                                {items.into_iter().map(|plan| {
                                    let is_latest = plan.season_year.is_some() && plan.season_year == latest_year;
                                    let (state_label, state_class) = season_state(plan.closed, is_latest);
                                    let note = plan.note;
                                    let note_view = (!note.is_empty()).then(|| view! {
                                        <small class="season-note">{note}</small>
                                    });
                                    let class = if plan.closed {
                                        "plan-list-item season-closed"
                                    } else if !is_latest {
                                        "plan-list-item season-older-open"
                                    } else {
                                        "plan-list-item season-latest"
                                    };
                                    view! {
                                        <A attr:class=class href=format!("/plans/{}", plan.id)>
                                            <span class="season-card-copy">
                                                <small class="season-year">{season_year_label(plan.season_year)}</small>
                                                <strong>{plan.name}</strong>
                                                {note_view}
                                                <span class="season-statuses">
                                                    <Show when=move || is_latest>
                                                        <small class="status latest-status">"ปีล่าสุด"</small>
                                                    </Show>
                                                    <small class=state_class>
                                                        {state_label}
                                                    </small>
                                                </span>
                                            </span>
                                            <span aria-hidden="true">"›"</span>
                                        </A>
                                    }
                                }).collect_view()}
                            </section>
                        }.into_any()
                    },
                    Err(_) => view! { <p class="form-message bad-message">"อ่านรายการฤดูกาลไม่ได้ กรุณาลองอีกครั้ง"</p> }.into_any(),
                })}
            </Suspense>

            <ActionForm action=logout>
                <button class="text-button" type="submit">"ออกจากระบบ"</button>
            </ActionForm>
        </section>
    }
}

#[component]
pub fn SeasonHistoryPage() -> impl IntoView {
    let history = Resource::new(|| (), |_| load_season_history());
    let query = use_query_map();
    let return_to = move || {
        query.with(|params| {
            params
                .get("from")
                .and_then(|value| value.parse::<i64>().ok())
        })
    };
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านประวัติฤดูกาล…"</p> }>
            {move || history.get().map(|result| match result {
                Ok(items) => view! { <SeasonHistoryView items return_to=return_to()/> }.into_any(),
                Err(_) => view! { <section class="card empty-state"><h1>"อ่านประวัติฤดูกาลไม่ได้"</h1><p>"กรุณาลองอีกครั้ง"</p><A attr:class="button secondary" href="/plans">"กลับฤดูกาลของฉัน"</A></section> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
pub fn SeasonHistoryView(items: Vec<SeasonHistoryItem>, return_to: Option<i64>) -> impl IntoView {
    use std::collections::HashMap;

    if items.is_empty() {
        return view! {
            <section class="page-stack history-page">
                <header class="page-heading compact-heading"><div><p class="eyebrow">"ผลจริงข้ามปี"</p><h1>"ประวัติฤดูกาล"</h1></div><A attr:class="icon-button" href="/plans" attr:aria-label="กลับฤดูกาลของฉัน">"×"</A></header>
                {history_return_link(return_to)}
                <section class="card empty-state"><h2>"ยังไม่มีฤดูกาลที่ปิดแล้ว"</h2><p>"เมื่อปิดฤดูกาลพร้อมผลจริง ประวัติจะเริ่มจากปีฐานตรงนี้"</p><A attr:class="button primary" href="/plans">"กลับฤดูกาลของฉัน"</A></section>
            </section>
        }.into_any();
    }

    let labels = items
        .iter()
        .map(|item| {
            (
                item.id,
                (
                    item.name.clone(),
                    item.actual_outcome
                        .as_ref()
                        .map(|actual| actual.outcome.note.clone())
                        .unwrap_or_default(),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let history = calc::build_season_history(
        items
            .into_iter()
            .map(|item| {
                let (actual, forecast) = item.actual_outcome.map_or((None, None), |record| {
                    if record.finalized {
                        (
                            Some(calc::analyze_actual(&record.outcome).metrics),
                            record.forecast,
                        )
                    } else {
                        (None, None)
                    }
                });
                calc::SeasonSnapshot {
                    season_id: item.id,
                    season_year: item.season_year,
                    actual,
                    forecast,
                }
            })
            .collect(),
    );

    view! {
        <section class="page-stack history-page">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">"ผลจริงข้ามปี"</p><h1>"ประวัติฤดูกาล"</h1></div>
                <A attr:class="icon-button" href="/plans" attr:aria-label="กลับฤดูกาลของฉัน">"×"</A>
            </header>
            {history_return_link(return_to)}
            <section class="card history-intro">
                <h2>"เทียบจากข้อมูลที่บันทึกจริง"</h2>
                <p>"ปีแรกที่มีผลจริงเป็นปีฐาน ไม่เรียกว่าแนวโน้ม ปีต่อไปเทียบกับฤดูกาลก่อนที่มีผลจริงเท่านั้น"</p>
                <p class="caption">"ผลต่างประมาณการ = ผลจริง - ประมาณการ · การเปลี่ยนแปลงข้ามปี = ((ผลจริงปีนี้ - ผลจริงปีก่อน) ÷ |ผลจริงปีก่อน|) × 100"</p>
            </section>
            {history.into_iter().map(|entry| {
                let (name, note) = labels.get(&entry.season.season_id).cloned().unwrap_or_default();
                view! { <HistorySeasonCard entry name note/> }
            }).collect_view()}
        </section>
    }.into_any()
}

fn history_return_link(return_to: Option<i64>) -> Option<impl IntoView> {
    return_to.map(|id| view! {
        <A attr:class="button secondary season-back" href=format!("/plans/{id}")>"‹ กลับไปฤดูกาลที่เปิดอยู่"</A>
    })
}

#[component]
fn HistorySeasonCard(entry: calc::SeasonHistoryEntry, name: String, note: String) -> impl IntoView {
    let year = entry.season.season_year;
    let year_label = season_year_label(year);
    let Some(actual) = entry.season.actual.clone() else {
        return view! {
            <article class="card history-season unavailable-history">
                <div class="section-title"><div><p class="eyebrow">{year_label}</p><h2>{name}</h2></div><span class="status muted">"ไม่มีผลจริง"</span></div>
                <p>"ฤดูกาลนี้ปิดก่อนมีขั้นตอนบันทึกผลจริง ระบบจึงไม่เติมศูนย์หรือสร้างแนวโน้มแทน"</p>
            </article>
        }.into_any();
    };

    let status = if entry.baseline {
        "ปีฐาน".to_owned()
    } else {
        entry.previous_actual_year.map_or_else(
            || "เทียบกับผลจริงก่อนหน้า".into(),
            |previous| format!("เทียบกับฤดูกาล {previous}"),
        )
    };
    let skipped = entry
        .previous_actual_year
        .zip(year)
        .is_some_and(|(previous, current)| current - previous > 1);
    let previous_year = entry.previous_actual_year;
    let trends = entry.actual_trends.clone();
    let forecast = entry.forecast_comparison.clone();
    let note_view = (!note.is_empty()).then(|| view! { <p class="history-note">{note}</p> });
    let cue_view = (!trends.is_empty()).then(|| {
        let cues = trends
            .clone()
            .into_iter()
            .map(|trend| view! { <TrendCue trend current_year=year previous_year/> })
            .collect_view();
        view! {
            <section class="history-cues" aria-label="ข้อสังเกตจากตัวเลข">
                <h3>"ข้อสังเกตจากกติกาคงที่"</h3>
                {cues}
            </section>
        }
    });

    view! {
        <article class="card history-season">
            <div class="section-title"><div><p class="eyebrow">{year_label}</p><h2>{name}</h2></div><span class="status muted">{status}</span></div>
            {note_view}
            <Show when=move || entry.baseline>
                <p class="baseline-copy">"นี่คือฤดูกาลแรกที่มีผลจริง จึงใช้เป็นปีฐานและยังไม่สรุปว่าเป็นแนวโน้ม"</p>
            </Show>
            <Show when=move || skipped>
                <p class="caption">"มีปีที่ข้ามระหว่างสองผลจริง ระบบเทียบเฉพาะปีที่แสดงและไม่ประมาณค่าปีที่หายไป"</p>
            </Show>
            {cue_view}
            <div class="table-scroll">
                <table class="history-table">
                    <caption>"ทุกค่ามาจากผลจริงและภาพนิ่งประมาณการของฤดูกาลนี้"</caption>
                    <thead><tr><th scope="col">"ตัวชี้วัด"</th><th scope="col">"ผลจริง"</th><th scope="col">"เทียบประมาณการ"</th><th scope="col">"เทียบฤดูก่อน"</th></tr></thead>
                    <tbody>
                        {ComparisonMetric::ALL.into_iter().map(|metric| view! {
                            <HistoryMetricRow metric actual=actual.clone() forecast=forecast.clone() trends=trends.clone() baseline=entry.baseline/>
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
            <A attr:class="button secondary" href=format!("/plans/{}/comparison", entry.season.season_id)>"ดูผลจริงเทียบประมาณการของฤดูนี้"</A>
        </article>
    }.into_any()
}

#[component]
fn TrendCue(
    trend: calc::MetricTrend,
    current_year: Option<i32>,
    previous_year: Option<i32>,
) -> impl IntoView {
    let (label, unit) = comparison_metric_label(trend.metric);
    let years = format!(
        "ฤดูกาล {} → {}",
        previous_year.map_or_else(|| "ก่อนหน้า".into(), |year| year.to_string()),
        current_year.map_or_else(|| "ปัจจุบัน".into(), |year| year.to_string())
    );
    let sentence = match (trend.direction, trend.percent_change) {
        (Some(calc::TrendDirection::Higher), Some(percent)) => {
            format!("{label} สูงกว่าฤดูกาลก่อน {}%", money(percent.abs()))
        }
        (Some(calc::TrendDirection::Lower), Some(percent)) => {
            format!("{label} ต่ำกว่าฤดูกาลก่อน {}%", money(percent.abs()))
        }
        (Some(calc::TrendDirection::Unchanged), Some(_)) => format!("{label} เท่ากับฤดูกาลก่อน"),
        _ => format!("{label} ยังคิดเปอร์เซ็นต์เปลี่ยนแปลงไม่ได้"),
    };
    let inputs = format!(
        "{years}: {} → {}",
        actual_metric(trend.previous, unit),
        actual_metric(trend.current, unit)
    );
    let formula = if trend.delta.is_some() && trend.percent_change.is_none() {
        "คิดเปอร์เซ็นต์ไม่ได้ เพราะค่าฤดูกาลก่อนเป็นศูนย์".to_owned()
    } else {
        "สูตร: ((ผลจริงปีนี้ - ผลจริงปีก่อน) ÷ |ผลจริงปีก่อน|) × 100".to_owned()
    };
    view! {
        <div class="cue-row"><strong>{sentence}</strong><span>{inputs}</span><small>{formula}</small></div>
    }
}

#[component]
fn HistoryMetricRow(
    metric: ComparisonMetric,
    actual: calc::OutcomeMetrics,
    forecast: Vec<calc::MetricComparison>,
    trends: Vec<calc::MetricTrend>,
    baseline: bool,
) -> impl IntoView {
    let (label, unit) = comparison_metric_label(metric);
    let actual_value = history_metric_value(&actual, metric);
    let forecast_delta = forecast
        .iter()
        .find(|row| row.metric == metric)
        .and_then(|row| row.delta);
    let trend = trends.iter().find(|trend| trend.metric == metric);
    let forecast_text = delta_text(forecast_delta, unit, "ประมาณการ");
    let trend_text = if baseline {
        "ปีฐาน".to_owned()
    } else if let Some(trend) = trend {
        match (trend.delta, trend.percent_change) {
            (Some(delta), Some(percent)) => format!(
                "{} {} {unit} ({}%)",
                direction_word(delta, "ฤดูก่อน"),
                money(delta.abs()),
                money(percent.abs())
            ),
            (Some(delta), None) => format!(
                "{} {} {unit} · ไม่มี % เพราะฐานเป็น 0",
                direction_word(delta, "ฤดูก่อน"),
                money(delta.abs())
            ),
            _ => "ยังเปรียบเทียบไม่ได้".into(),
        }
    } else {
        "ยังเปรียบเทียบไม่ได้".into()
    };
    view! { <tr><th scope="row">{label}</th><td>{actual_metric(actual_value, unit)}</td><td>{forecast_text}</td><td>{trend_text}</td></tr> }
}

fn history_metric_value(
    metrics: &calc::OutcomeMetrics,
    metric: ComparisonMetric,
) -> Option<rust_decimal::Decimal> {
    match metric {
        ComparisonMetric::SellableYieldKg => metrics.sellable_yield_kg,
        ComparisonMetric::Revenue => metrics.revenue,
        ComparisonMetric::TotalCost => metrics.total_cost,
        ComparisonMetric::Profit => metrics.profit,
        ComparisonMetric::AveragePricePerKg => metrics.average_price_per_kg,
        ComparisonMetric::CostPerKg => metrics.cost_per_kg,
    }
}

fn delta_text(delta: Option<rust_decimal::Decimal>, unit: &str, comparison: &str) -> String {
    delta.map_or_else(
        || "ยังเปรียบเทียบไม่ได้".into(),
        |delta| {
            format!(
                "{} {} {unit}",
                direction_word(delta, comparison),
                money(delta.abs())
            )
        },
    )
}

fn direction_word(delta: rust_decimal::Decimal, comparison: &str) -> String {
    if delta > rust_decimal::Decimal::ZERO {
        format!("สูงกว่า{comparison}")
    } else if delta < rust_decimal::Decimal::ZERO {
        format!("ต่ำกว่า{comparison}")
    } else {
        format!("เท่ากับ{comparison}")
    }
}

#[component]
pub fn NewSeasonPage() -> impl IntoView {
    let query = use_query_map();
    let source_id = move || {
        query.with(|params| {
            params
                .get("source")
                .and_then(|value| value.parse::<i64>().ok())
        })
    };
    let source = Resource::new(source_id, |id| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });

    view! {
        <Suspense fallback=move || view! { <p>"กำลังเตรียมฤดูกาล…"</p> }>
            {move || source.get().map(|result| match result {
                Ok(record) => view! { <NewSeasonForm source=record/> }.into_any(),
                Err(_) => view! { <section class="card"><h1>"เตรียมฤดูกาลไม่ได้"</h1><A href="/plans">"กลับไปรายการฤดูกาล"</A></section> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn NewSeasonForm(source: Option<PlanRecord>) -> impl IntoView {
    let create = ServerAction::<CreateSeason>::new();
    let source_id = source.as_ref().map(|record| record.id);
    let year = source
        .as_ref()
        .and_then(|record| record.season_year)
        .map(|year| year + 1)
        .map(|year| year.to_string())
        .unwrap_or_default();
    let name = source
        .as_ref()
        .map(|record| record.form.name.clone())
        .unwrap_or_default();
    let source_name = source.as_ref().map(|record| record.form.name.clone());

    view! {
        <section class="page-stack">
            <header class="page-heading">
                <div>
                    <p class="eyebrow">"ฤดูกาล"</p>
                    <h1>{if source_id.is_some() { "ทำฤดูกาลถัดไป" } else { "เริ่มฤดูกาลใหม่" }}</h1>
                </div>
                <A attr:class="icon-button" href="/plans" attr:aria-label="ยกเลิกและกลับไปรายการฤดูกาล">"×"</A>
            </header>
            <section class="card season-create-card">
                {source_name.map(|name| view! { <p class="copy-source">"คัดลอกข้อมูลตั้งต้นจาก " <strong>{name}</strong></p> })}
                <p>"หนึ่งฤดูกาลรวมประมาณการของทุกแปลงในปีเก็บเกี่ยวเดียว"</p>
                <ActionForm action=create>
                    {source_id.map(|id| view! { <input type="hidden" name="source_id" value=id/> })}
                    <label>
                        <span>"ปีฤดูกาล (พ.ศ.)"</span>
                        <input id="season-year" type="text" name="season_year" inputmode="numeric" pattern="[0-9]{4}" maxlength="4" value=year placeholder="เช่น 2569" aria-describedby="season-year-help" required/>
                        <small id="season-year-help" class="field-hint">"ปี พ.ศ. ที่จะเก็บเกี่ยว เช่น 2569 ใช้แยกฤดูกาลในประวัติ"</small>
                    </label>
                    <label>
                        <span>"ชื่อฤดูกาล"</span>
                        <input id="season-name" type="text" name="name" maxlength="120" value=name placeholder="เช่น สวนรวม หมอนทอง" aria-describedby="season-name-help" required/>
                        <small id="season-name-help" class="field-hint">"ชื่อสั้น ๆ ที่คุณจำได้เมื่อกลับมาดู เช่น สวนรวม หมอนทอง"</small>
                    </label>
                    <label>
                        <span>"บันทึก (ไม่บังคับ)"</span>
                        <textarea id="season-note" name="note" maxlength="2000" rows="4" placeholder="เช่น ปีแรกที่เปลี่ยนปุ๋ย" aria-describedby="season-note-help"></textarea>
                        <small id="season-note-help" class="field-hint">"เก็บเหตุการณ์สำคัญไว้เทียบกับผลจริงภายหลัง เว้นว่างได้"</small>
                    </label>
                    <button class="primary" type="submit">"สร้างฤดูกาล"</button>
                </ActionForm>
                <ServerErrorMessage action=create/>
            </section>
        </section>
    }
}

#[component]
pub fn DemoPage() -> impl IntoView {
    let form = RwSignal::new(PlanForm::from_plan(&calc::workbook_sample()));
    view! {
        <section class="page-stack demo-page">
            <header class="page-heading">
                <div><p class="eyebrow">"โหมดตัวอย่าง"</p><h1>"ลองก่อน โดยไม่บันทึก"</h1></div>
                <A attr:class="icon-button" href="/plans" attr:aria-label="ปิดตัวอย่าง">"×"</A>
            </header>
            <div class="demo-banner" role="status">
                <strong>"ข้อมูลนี้ไม่ใช่ฤดูกาลของคุณ"</strong>
                <span>"ลองเปลี่ยนตัวเลขได้ ระบบจะไม่บันทึกลงประวัติ"</span>
            </div>
            <section class="card demo-controls">
                <h2>"ลองเปลี่ยนผลผลิตและราคา"</h2>
                <PlanField
                    field_id="demo-fruits"
                    label="ผลต่อต้น"
                    formal_term="ผลผลิตต่อต้น"
                    hint="ลองเปลี่ยนจำนวนผลเฉลี่ยจากต้นที่ให้ผลแล้ว"
                    example="ตัวอย่าง 40 ผลต่อต้น"
                    outcome="ค่านี้เปลี่ยนผลผลิต รายได้ และกำไรตัวอย่างด้านล่าง"
                    unit="ผล"
                    numeric=true
                    value=Signal::derive(move || form.get().production.fruits_per_tree)
                    on_value=Callback::new(move |value| form.update(|form| form.production.fruits_per_tree = value))
                    closed=false
                />
                <PlanField
                    field_id="demo-price"
                    label="ราคาเกรดแรก"
                    formal_term="ราคาขายต่อกิโลกรัม"
                    hint="ลองใช้ราคาที่คาดว่าจะขายเกรดแรกได้"
                    example="ตัวอย่าง 95 บาทต่อกิโลกรัม"
                    outcome="ค่านี้เปลี่ยนราคาเฉลี่ย รายได้ และกำไรตัวอย่างด้านล่าง"
                    unit="บาท/กก."
                    numeric=true
                    value=Signal::derive(move || form.get().grades.first().map(|grade| grade.price_per_kg.clone()).unwrap_or_default())
                    on_value=Callback::new(move |value| form.update(|form| if let Some(grade) = form.grades.first_mut() { grade.price_per_kg = value }))
                    closed=false
                />
                <div class="actions">
                    <button class="secondary" type="button" on:click=move |_| form.set(PlanForm::from_plan(&calc::workbook_sample()))>"คืนค่าตัวอย่าง"</button>
                    <A attr:class="button primary" href="/plans/new">"สร้างฤดูกาลของฉัน"</A>
                </div>
            </section>
            <section aria-live="polite">
                {move || {
                    let form = form.get();
                    form.to_plan().ok().map(|plan| view! {
                        <crate::analysis_ui::DashboardFigures id=None analysis=calc::analyze(&plan) decisions=form.decision_readiness(&[], None)/>
                    })
                }}
            </section>
        </section>
    }
}

#[component]
pub fn PlanHubPage() -> impl IntoView {
    let id = route_plan_id();
    let record = Resource::new(id, |id| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านฤดูกาล…"</p> }>
            {move || record.get().map(|result| match result {
                Ok(Some(record)) => view! { <PlanHub record/> }.into_any(),
                _ => view! { <section class="card"><h1>"ไม่พบฤดูกาลนี้"</h1><A href="/plans">"กลับไปฤดูกาลของฉัน"</A></section> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
pub fn PlanHub(record: PlanRecord) -> impl IntoView {
    let update_metadata = ServerAction::<UpdateSeasonMetadata>::new();
    let id = record.id;
    let form = record.form;
    let name = form.name.clone();
    let year = record.season_year;
    let note = record.note;
    let closed = record.closed;
    let forecast_mode = record.forecast_mode;
    let quick_estimate = record.quick_estimate;
    let asset_allocations = record.asset_allocations;
    let starting_capital = record.starting_capital;

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">{season_year_label(year)}</p><h1>{name.clone()}</h1></div>
                <A attr:class="icon-button" href=format!("/plans?from={id}") attr:aria-label="เปิดรายการฤดูกาล">"×"</A>
            </header>
            <Show when=move || closed>
                <div class="closed-banner" role="status">"ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"</div>
            </Show>
            <section class="card season-details">
                <h2>"รายละเอียดฤดูกาล"</h2>
                {if closed {
                    view! {
                        <dl class="season-metadata">
                            <div><dt>"ปีฤดูกาล"</dt><dd>{season_year_label(year)}</dd></div>
                            <div><dt>"ชื่อ"</dt><dd>{name.clone()}</dd></div>
                            <div><dt>"บันทึก"</dt><dd>{if note.is_empty() { "—".into() } else { note.clone() }}</dd></div>
                        </dl>
                    }.into_any()
                } else {
                    view! {
                        <ActionForm action=update_metadata>
                            <input type="hidden" name="id" value=id/>
                            <label><span>"ปีฤดูกาล (พ.ศ.)"</span><input type="text" name="season_year" inputmode="numeric" pattern="[0-9]{4}" maxlength="4" value=year.map(|year| year.to_string()).unwrap_or_default() required/></label>
                            <label><span>"ชื่อฤดูกาล"</span><input type="text" name="name" maxlength="120" value=name.clone() required/></label>
                            <label><span>"บันทึก (ไม่บังคับ)"</span><textarea name="note" maxlength="2000" rows="3">{note.clone()}</textarea></label>
                            <button class="secondary" type="submit">"บันทึกรายละเอียด"</button>
                        </ActionForm>
                        <ServerMessage action=update_metadata/>
                    }.into_any()
                }}
            </section>
            {match forecast_mode {
                calc::ForecastMode::Quick => view! {
                    <QuickModeHub id estimate=quick_estimate.clone() closed/>
                }.into_any(),
                calc::ForecastMode::Detailed => view! {
                    <DetailedModeHub id form=form.clone() closed asset_depreciation=included_asset_depreciation(&asset_allocations) decisions=form.decision_readiness(&asset_allocations, starting_capital)/>
                }.into_any(),
            }}
            // No live total here. The bar exists so a figure moves while the
            // owner types, and nothing on this page is typed; it only covered
            // the cards below it. The dashboard carries the figure instead.
            <section class="card plan-actions">
                <h2>"จัดการฤดูกาล"</h2>
                <A attr:class="button secondary" href=format!("/plans/new?source={id}")>"ทำฤดูกาลถัดไปจากฤดูนี้"</A>
                {if closed {
                    if record.actual_outcome.as_ref().is_some_and(|actual| actual.finalized) {
                        view! { <A attr:class="button primary" href=format!("/plans/{id}/comparison")>"ดูผลจริงเทียบประมาณการ"</A> }.into_any()
                    } else {
                        view! { <p class="caption">"ฤดูกาลนี้ปิดก่อนมีการบันทึกผลจริง จึงไม่มีตัวเลขเปรียบเทียบ"</p> }.into_any()
                    }
                } else {
                    view! { <A attr:class="button danger" href=format!("/plans/{id}/close")>"บันทึกผลจริงและปิดฤดูกาล"</A> }.into_any()
                }}
            </section>
            <BottomNav plan_id=id active=NavSection::Input/>
        </section>
    }
}

#[component]
pub fn ActualClosePage() -> impl IntoView {
    let id = route_plan_id();
    let record = Resource::new(id, |id| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านฤดูกาล…"</p> }>
            {move || record.get().map(|result| match result {
                Ok(Some(record)) => view! { <ActualCloseView record/> }.into_any(),
                _ => not_found_view(),
            })}
        </Suspense>
    }
}

#[component]
pub fn ActualCloseView(record: PlanRecord) -> impl IntoView {
    let save = ServerAction::<SaveActualDraft>::new();
    let id = record.id;
    let season = format!(
        "{} · {}",
        season_year_label(record.season_year),
        record.form.name
    );
    let draft = record
        .actual_outcome
        .as_ref()
        .map(|actual| actual.outcome.clone())
        .unwrap_or_default();

    if record.closed {
        return view! {
            <section class="page-stack actual-close-page">
                <header class="page-heading compact-heading"><div><p class="eyebrow">{season}</p><h1>"ฤดูกาลนี้ปิดแล้ว"</h1></div></header>
                {if record.actual_outcome.as_ref().is_some_and(|actual| actual.finalized) {
                    view! { <A attr:class="button primary" href=format!("/plans/{id}/comparison")>"ดูผลจริงเทียบประมาณการ"</A> }.into_any()
                } else {
                    view! { <section class="card empty-state"><p>"ฤดูกาลนี้ปิดก่อนมีการบันทึกผลจริง จึงไม่มีตัวเลขเปรียบเทียบ"</p><A href=format!("/plans/{id}")>"กลับหน้าฤดูกาล"</A></section> }.into_any()
                }}
            </section>
        }
        .into_any();
    }

    view! {
        <section class="page-stack actual-close-page">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">{season}</p><h1>"บันทึกผลจริง"</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="ยกเลิกและกลับหน้าฤดูกาล">"×"</A>
            </header>
            <section class="card actual-entry-card">
                <p>"กรอกยอดรวมเมื่อจบฤดู ระบบจะให้ตรวจทานอีกครั้งก่อนปิดถาวร"</p>
                <ActionForm action=save>
                    <input type="hidden" name="id" value=id/>
                    <label><span>"ขายได้จริงทั้งหมดเท่าไร"</span><small class="formal-term">"ผลผลิตที่ขายได้จริง"</small><span class="input-with-unit"><input id="actual-yield" type="text" name="sellable_yield_kg" inputmode="decimal" value=decimal_input(draft.sellable_yield_kg) aria-describedby="actual-yield-help" required autofocus/><span class="unit">"กก."</span></span><small id="actual-yield-help" class="field-hint">"ดูจากยอดส่งขายหรือสรุปน้ำหนักหลังหักผลเสีย ใช้คำนวณราคาขายจริงต่อกิโลกรัม"</small></label>
                    <label><span>"รับเงินจากการขายรวมเท่าไร"</span><small class="formal-term">"รายได้จริง"</small><span class="input-with-unit"><input id="actual-revenue" type="text" name="revenue" inputmode="decimal" value=decimal_input(draft.revenue) aria-describedby="actual-revenue-help" required/><span class="unit">"บาท"</span></span><small id="actual-revenue-help" class="field-hint">"ยอดรับรวมก่อนหักค่าใช้จ่าย ใช้คำนวณกำไรหรือขาดทุนจริง"</small></label>
                    <label><span>"จ่ายค่าใช้จ่ายทั้งฤดูรวมเท่าไร"</span><small class="formal-term">"ต้นทุนรวมจริง"</small><span class="input-with-unit"><input id="actual-cost" type="text" name="total_cost" inputmode="decimal" value=decimal_input(draft.total_cost) aria-describedby="actual-cost-help" required/><span class="unit">"บาท"</span></span><small id="actual-cost-help" class="field-hint">"รวมค่าใช้จ่ายที่จ่ายจริงและต้นทุนที่ต้องการนับ ใช้หักจากรายได้จริง"</small></label>
                    <label><span>"บันทึกว่าเกิดอะไรขึ้น (ไม่บังคับ)"</span><textarea id="actual-note" name="note" maxlength="2000" rows="4" aria-describedby="actual-note-help">{draft.note}</textarea><small id="actual-note-help" class="field-hint">"เช่น ผลผลิตลดเพราะฝน หรือราคาดีกว่าที่คาด เพื่อช่วยอ่านผลเทียบภายหลัง"</small></label>
                    <button class="primary" type="submit">"บันทึกและตรวจทาน"</button>
                </ActionForm>
                <ServerErrorMessage action=save/>
            </section>
            <A attr:class="button secondary" href=format!("/plans/{id}")>"ยกเลิก · เก็บฤดูกาลไว้เปิดอยู่"</A>
        </section>
    }
    .into_any()
}

#[component]
pub fn ActualReviewPage() -> impl IntoView {
    let id = route_plan_id();
    let record = Resource::new(id, |id| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านผลจริง…"</p> }>
            {move || record.get().map(|result| match result {
                Ok(Some(record)) => view! { <ActualReviewView record/> }.into_any(),
                _ => not_found_view(),
            })}
        </Suspense>
    }
}

#[component]
pub fn ActualReviewView(record: PlanRecord) -> impl IntoView {
    let finalize = ServerAction::<FinalizeActual>::new();
    let id = record.id;
    let season = format!(
        "{} · {}",
        season_year_label(record.season_year),
        record.form.name
    );
    let Some(actual_record) = record.actual_outcome.clone() else {
        return view! {
            <section class="page-stack actual-review-page"><section class="card empty-state"><h1>"ยังไม่มีผลจริงให้ตรวจทาน"</h1><A attr:class="button primary" href=format!("/plans/{id}/close")>"กรอกผลจริง"</A></section></section>
        }.into_any();
    };
    if actual_record.finalized || record.closed {
        return view! {
            <section class="page-stack actual-review-page"><section class="card"><h1>"ฤดูกาลนี้ปิดแล้ว"</h1><A attr:class="button primary" href=format!("/plans/{id}/comparison")>"ดูผลจริงเทียบประมาณการ"</A></section></section>
        }.into_any();
    }
    let actual = actual_record.outcome;
    let analysis = calc::analyze_actual(&actual);
    if !analysis.input_issues.is_empty() {
        return view! {
            <section class="page-stack actual-review-page"><section class="card empty-state"><h1>"ผลจริงยังไม่ครบ"</h1><p>"กลับไปกรอกผลผลิต รายได้ และต้นทุนให้ครบก่อนปิดฤดูกาล"</p><A attr:class="button primary" href=format!("/plans/{id}/close")>"กลับไปแก้"</A></section></section>
        }.into_any();
    }

    view! {
        <section class="page-stack actual-review-page">
            <header class="page-heading compact-heading"><div><p class="eyebrow">{season}</p><h1>"ตรวจทานก่อนปิดฤดูกาล"</h1></div><A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="ยกเลิกและกลับหน้าฤดูกาล">"×"</A></header>
            <section class="card">
                <h2>"ข้อมูลที่กำลังจะล็อก"</h2>
                <dl class="season-metadata">
                    <div><dt>"ผลผลิตที่ขายได้จริง"</dt><dd>{format!("{} กก.", money(actual.sellable_yield_kg.expect("complete actual yield")))}</dd></div>
                    <div><dt>"รายได้จริง"</dt><dd>{format!("{} บาท", money(actual.revenue.expect("complete actual revenue")))}</dd></div>
                    <div><dt>"ต้นทุนรวมจริง"</dt><dd>{format!("{} บาท", money(actual.total_cost.expect("complete actual cost")))}</dd></div>
                    <div><dt>"กำไรหรือขาดทุนจริง"</dt><dd>{format!("{} บาท", money(analysis.metrics.profit.expect("complete actual profit")))}</dd></div>
                    <div><dt>"บันทึก"</dt><dd>{if actual.note.is_empty() { "—".into() } else { actual.note.clone() }}</dd></div>
                </dl>
            </section>
            <section class="card confirm-box">
                <h2>"ยืนยันครั้งสุดท้าย"</h2>
                <p>"เมื่อยืนยันแล้ว ผลจริงและประมาณการ ณ ตอนนี้จะถูกเก็บเป็นภาพนิ่ง ฤดูกาลนี้จะแก้ไขไม่ได้"</p>
                <ActionForm action=finalize><input type="hidden" name="id" value=id/><button class="danger" type="submit">"ยืนยันผลจริงและปิดฤดูกาล"</button></ActionForm>
                <ServerErrorMessage action=finalize/>
            </section>
            <A attr:class="button secondary" href=format!("/plans/{id}/close")>"กลับไปแก้ผลจริง"</A>
            <A attr:class="button secondary" href=format!("/plans/{id}")>"ยกเลิก · ยังไม่ปิดฤดูกาล"</A>
        </section>
    }.into_any()
}

#[component]
pub fn ActualComparisonPage() -> impl IntoView {
    let id = route_plan_id();
    let record = Resource::new(id, |id| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังเปรียบเทียบ…"</p> }>
            {move || record.get().map(|result| match result {
                Ok(Some(record)) => view! { <ActualComparisonView record/> }.into_any(),
                _ => not_found_view(),
            })}
        </Suspense>
    }
}

#[component]
pub fn ActualComparisonView(record: PlanRecord) -> impl IntoView {
    let id = record.id;
    let season = format!(
        "{} · {}",
        season_year_label(record.season_year),
        record.form.name
    );
    let Some(actual_record) = record.actual_outcome else {
        return legacy_actual_unavailable(id, season);
    };
    let (Some(forecast), Some(mode)) = (actual_record.forecast, actual_record.forecast_mode) else {
        return legacy_actual_unavailable(id, season);
    };
    if !record.closed || !actual_record.finalized {
        return view! { <section class="page-stack"><section class="card empty-state"><h1>"ฤดูกาลนี้ยังไม่ปิด"</h1><A attr:class="button primary" href=format!("/plans/{id}/close/review")>"กลับไปตรวจทาน"</A></section></section> }.into_any();
    }
    let actual = actual_record.outcome;
    let actual_metrics = calc::analyze_actual(&actual).metrics;
    let comparisons = calc::compare(&forecast, &actual);

    view! {
        <section class="page-stack actual-comparison-page">
            <header class="page-heading compact-heading"><div><p class="eyebrow">{season}</p><h1>"ผลจริงเทียบประมาณการ"</h1></div><A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A></header>
            <section class="card actual-result-card">
                <p class="eyebrow">"ผลจริง"</p>
                <p class="result-label">{if actual_metrics.profit.is_some_and(|value| value >= rust_decimal::Decimal::ZERO) { "กำไรจริง" } else { "ขาดทุนจริง" }}</p>
                <strong class="hero-value">{format!("{} บาท", money(actual_metrics.profit.unwrap_or_default().abs()))}</strong>
                <dl class="season-metadata">
                    <div><dt>"ผลผลิตที่ขายได้"</dt><dd>{actual_metric(actual_metrics.sellable_yield_kg, "กก.")}</dd></div>
                    <div><dt>"รายได้"</dt><dd>{actual_metric(actual_metrics.revenue, "บาท")}</dd></div>
                    <div><dt>"ต้นทุนรวม"</dt><dd>{actual_metric(actual_metrics.total_cost, "บาท")}</dd></div>
                    <div><dt>"ราคาขายเฉลี่ย"</dt><dd>{actual_metric(actual_metrics.average_price_per_kg, "บาท/กก.")}</dd></div>
                    <div><dt>"ต้นทุนต่อกิโลกรัม"</dt><dd>{actual_metric(actual_metrics.cost_per_kg, "บาท/กก.")}</dd></div>
                </dl>
                <p class="caption">{if actual.note.is_empty() { "ไม่มีบันทึกผลจริง".into() } else { actual.note.clone() }}</p>
            </section>
            <section class="comparison-list" aria-label="รายการเปรียบเทียบผลจริงกับประมาณการ">
                <div class="section-title"><div><p class="eyebrow">"สูตรทุกแถว: ผลจริง - ประมาณการ"</p><h2>"ต่างจากที่วางไว้เท่าไร"</h2><p>{format!("แหล่งประมาณการ: {}", forecast_mode_label(mode))}</p></div></div>
                {comparisons.into_iter().map(comparison_card).collect_view()}
            </section>
            <A attr:class="button secondary" href=format!("/plans/{id}")>"กลับหน้าฤดูกาล"</A>
        </section>
    }.into_any()
}

fn legacy_actual_unavailable(id: i64, season: String) -> AnyView {
    view! {
        <section class="page-stack actual-comparison-page">
            <header class="page-heading compact-heading"><div><p class="eyebrow">{season}</p><h1>"ผลจริงเทียบประมาณการ"</h1></div></header>
            <section class="card empty-state"><h2>"ไม่มีผลจริงที่บันทึกไว้"</h2><p>"ฤดูกาลนี้ปิดก่อนมีขั้นตอนบันทึกผลจริง ระบบจึงไม่เติมศูนย์หรือสร้างตัวเลขแทน"</p><A attr:class="button secondary" href=format!("/plans/{id}")>"กลับหน้าฤดูกาล"</A></section>
        </section>
    }.into_any()
}

fn comparison_card(row: calc::MetricComparison) -> impl IntoView {
    let (label, unit) = comparison_metric_label(row.metric);
    let result = row.delta.map_or_else(
        || "ยังเปรียบเทียบไม่ได้".into(),
        |delta| {
            let direction = if delta > rust_decimal::Decimal::ZERO {
                "สูงกว่าประมาณการ"
            } else if delta < rust_decimal::Decimal::ZERO {
                "ต่ำกว่าประมาณการ"
            } else {
                "เท่ากับประมาณการ"
            };
            format!("{direction} {} {unit}", money(delta.abs()))
        },
    );
    view! {
        <article class="card comparison-card">
            <h3>{label}</h3><strong>{result}</strong>
            <dl class="comparison-values">
                <div><dt>"ประมาณการ"</dt><dd>{actual_metric(row.forecast, unit)}</dd></div>
                <div><dt>"ผลจริง"</dt><dd>{actual_metric(row.actual, unit)}</dd></div>
            </dl>
            <p class="caption">"ผลต่าง = ผลจริง - ประมาณการ"</p>
        </article>
    }
}

fn comparison_metric_label(metric: ComparisonMetric) -> (&'static str, &'static str) {
    match metric {
        ComparisonMetric::SellableYieldKg => ("ผลผลิตที่ขายได้", "กก."),
        ComparisonMetric::Revenue => ("รายได้", "บาท"),
        ComparisonMetric::TotalCost => ("ต้นทุนรวม", "บาท"),
        ComparisonMetric::Profit => ("กำไรหรือขาดทุน", "บาท"),
        ComparisonMetric::AveragePricePerKg => ("ราคาขายเฉลี่ย", "บาท/กก."),
        ComparisonMetric::CostPerKg => ("ต้นทุนต่อกิโลกรัม", "บาท/กก."),
    }
}

fn actual_metric(value: Option<rust_decimal::Decimal>, unit: &str) -> String {
    value.map_or_else(
        || "ยังไม่มีข้อมูล".into(),
        |value| format!("{} {unit}", money(value)),
    )
}

fn forecast_mode_label(mode: calc::ForecastMode) -> &'static str {
    match mode {
        calc::ForecastMode::Quick => "ประมาณการเร็ว",
        calc::ForecastMode::Detailed => "แผนละเอียด",
    }
}

fn decimal_input(value: Option<rust_decimal::Decimal>) -> String {
    value
        .map(|value| value.normalize().to_string())
        .unwrap_or_default()
}

fn not_found_view() -> AnyView {
    view! { <section class="card"><h1>"ไม่พบฤดูกาลนี้"</h1><A href="/plans">"กลับไปฤดูกาลของฉัน"</A></section> }.into_any()
}

#[component]
fn QuickModeHub(id: i64, estimate: calc::QuickEstimate, closed: bool) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
    let complete = estimate.first_incomplete().is_none();
    let answered = calc::QuickInputField::ALL
        .into_iter()
        .filter(|field| {
            estimate
                .value(*field)
                .is_some_and(|value| value > rust_decimal::Decimal::ZERO)
        })
        .count();
    let destination = quick_resume_path(id, &estimate);

    view! {
        <section class="card quick-mode-card">
            <div class="section-title">
                <div>
                    <p class="eyebrow">"โหมดที่ใช้อยู่"</p>
                    <h2>"ประมาณการเร็ว"</h2>
                    <p>"ตอบ 3 ข้อเพื่อดูรายได้ กำไร ต้นทุนต่อกิโลกรัม และราคาคุ้มทุน"</p>
                </div>
                <strong>{format!("{answered}/3")}</strong>
            </div>
            <A attr:class="button primary" href=destination>
                {if complete { "ดูผลประมาณการ" } else { "ทำประมาณการต่อ" }}
            </A>
            <Show when=move || !closed>
                <ActionForm action=switch>
                    <input type="hidden" name="id" value=id/>
                    <input type="hidden" name="mode" value="detailed"/>
                    <button class="text-button" type="submit">"เปลี่ยนเป็นแผนละเอียด"</button>
                </ActionForm>
            </Show>
            <ServerErrorMessage action=switch/>
        </section>
    }
}

#[component]
fn DetailedModeHub(
    id: i64,
    form: PlanForm,
    closed: bool,
    asset_depreciation: Option<rust_decimal::Decimal>,
    decisions: Vec<DecisionReadiness>,
) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
    let next = ["production", "expenses", "variable-costs", "fixed-costs"]
        .into_iter()
        .find(|section| form.section_readiness(section).tone == ReadinessTone::Missing);
    let (guide_title, guide_body, guide_href, guide_action) = match next {
        Some("expenses") => (
            "บอกว่าค่าใช้จ่ายที่จดไว้เป็นแบบไหน",
            "ตอบคำถามสั้น ๆ ทีละรายการ ระบบจะย้ายไปส่วนที่ถูกให้ ตัวเลขยังไม่ถูกนับจนกว่าจะตอบ",
            format!("/plans/{id}/expenses"),
            "จัดประเภทค่าใช้จ่าย",
        ),
        Some("production") => (
            "เริ่มจากยอดที่จะขายและราคาขาย",
            "ข้อมูลส่วนนี้ทำให้ระบบคำนวณรายได้โดยประมาณได้",
            format!("/plans/{id}/production"),
            "กรอกผลผลิตและราคา",
        ),
        Some("variable-costs") => (
            "ต่อด้วยค่าใช้จ่ายที่เพิ่มเมื่อทำหรือขายมากขึ้น",
            "เช่น ปุ๋ย ยา คนเก็บ ขนส่ง และกล่อง ถ้าจำได้แต่ไม่รู้ประเภท จดไว้ที่ค่าใช้จ่ายที่จำได้ก่อนได้ หรือยืนยันว่าไม่มีถ้าไม่มีจริง",
            format!("/plans/{id}/variable-costs"),
            "กรอกค่าใช้จ่ายตามการผลิต",
        ),
        Some("fixed-costs") => (
            "แล้วค่าใช้จ่ายที่ยังมีแม้ไม่มีทุเรียนขาย",
            "เช่น ค่าเช่า เงินเดือนประจำ ดอกเบี้ย ถ้าไม่มีจริง ยืนยันว่าไม่มีได้เลย",
            format!("/plans/{id}/fixed-costs"),
            "กรอกค่าใช้จ่ายประจำ",
        ),
        _ => (
            "ตัวเลขหลักพร้อมแล้ว",
            "ดูรายได้ ต้นทุน และกำไรโดยประมาณ แล้วกลับมาแก้ส่วนใดก็ได้",
            format!("/plans/{id}/dashboard"),
            "ดูผลสรุป",
        ),
    };
    view! {
        <section class="card detailed-mode-card">
            <p class="eyebrow">"โหมดที่ใช้อยู่"</p>
            <h2>"แผนละเอียด"</h2>
            <p>"คำนวณจากข้อมูลตลาด ผลผลิต เกรด และรายการต้นทุนด้านล่าง"</p>
            <Show when=move || !closed>
                <ActionForm action=switch>
                    <input type="hidden" name="id" value=id/>
                    <input type="hidden" name="mode" value="quick"/>
                    <button class="text-button" type="submit">"ใช้ประมาณการเร็ว"</button>
                </ActionForm>
            </Show>
            <ServerErrorMessage action=switch/>
        </section>
        <section class="card guided-next">
            <p class="eyebrow">"ขั้นที่แนะนำ"</p>
            <h2>{guide_title}</h2>
            <p>{guide_body}</p>
            <A attr:class="button primary" href=guide_href>{guide_action}</A>
            <p class="caption">"คุณยังเปิดดูหรือแก้ส่วนอื่นด้านล่างได้ตลอด ระบบไม่ล็อกลำดับ"</p>
        </section>
        <DecisionList plan_id=id decisions compact=true/>
        <div class="overview-heading">
            <h2>"ดูและแก้ข้อมูลทั้งหมด"</h2>
            <p>"เลือกเฉพาะส่วนที่ต้องการได้ คำสถานะบอกว่าตอนนี้คำนวณอะไรได้แล้ว"</p>
        </div>
        <section class="section-list">
            <A attr:class="section-card" href=format!("/plans/{id}/dashboard")>
                <span><strong>"หน้าแรก"</strong><small>"ตัวเลขสรุปของฤดูกาลนี้"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
            <A attr:class="section-card" href=format!("/plans/{id}/analysis")>
                <span><strong>"วิเคราะห์"</strong><small>"ประสิทธิภาพ ตรวจสอบ ภาษี สถานการณ์"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
            {MAIN_SECTIONS.into_iter().map(|(slug, title, description)| {
                let readiness = form.section_readiness_with_assets(slug, asset_depreciation);
                let status_class = match readiness.tone {
                    ReadinessTone::Ready => "status good",
                    ReadinessTone::Missing => "status warning",
                    ReadinessTone::Optional => "status muted",
                };
                view! {
                    <A attr:class="section-card" href=format!("/plans/{id}/{slug}")>
                        <span><strong>{title}</strong><small>{description}</small></span>
                        <span class=status_class>{readiness.label}</span>
                    </A>
                }
            }).collect_view()}
        </section>
        <details class="card advanced-planning">
            <summary>"การวางแผนขั้นสูง (ไม่บังคับ)"</summary>
            <p>"ตั้งเป้าหมาย KPI ของสวนเองเมื่ออยากใช้เกณฑ์เปรียบเทียบ ส่วนนี้ไม่ขวางผลประมาณการหรือความพร้อมของแผน"</p>
            <A attr:class="section-card" href=format!("/plans/{id}/targets")>
                <span><strong>"เป้าหมาย KPI"</strong><small>"ค่าที่ตั้งไว้เดิมยังอยู่และแก้ได้ที่นี่"</small></span>
                <span aria-hidden="true">"›"</span>
            </A>
            <A attr:class="section-card" href=format!("/plans/{id}/assets")>
                <span><strong>"ของที่ใช้หลายปี และเงินก้อนที่ลงไป"</strong><small>"สินทรัพย์ ค่าเสื่อม เงินลงทุน · เลือกว่าจะรวมในฤดูนี้หรือไม่"</small></span>
                <span aria-hidden="true">"›"</span>
            </A>
        </details>
    }
}

#[component]
pub fn QuickPlanRoute() -> impl IntoView {
    let params = use_params_map();
    let key = move || {
        params.with(|params| {
            let id = params.get("id").and_then(|id| id.parse::<i64>().ok());
            let step = params.get("step").unwrap_or_default();
            (id, step)
        })
    };
    let record = Resource::new(key, |(id, _)| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });

    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านประมาณการ…"</p> }>
            {move || record.get().map(|result| {
                let (_, step) = key();
                match result {
                    Ok(Some(record)) if record.forecast_mode == calc::ForecastMode::Detailed =>
                        view! { <DetailedModeActiveNotice record/> }.into_any(),
                    Ok(Some(record)) if step == "result" || record.closed =>
                        view! { <QuickResultView record/> }.into_any(),
                    Ok(Some(record)) if matches!(step.as_str(), "production" | "price" | "cost") =>
                        view! { <QuickQuestionView record step/> }.into_any(),
                    _ => view! { <section class="card"><h1>"ไม่พบขั้นตอนนี้"</h1><A href="/plans">"กลับไปฤดูกาลของฉัน"</A></section> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn QuickQuestionView(record: PlanRecord, step: String) -> impl IntoView {
    let save = ServerAction::<SaveQuickStep>::new();
    let id = record.id;
    let (position, question, field_label, hint, unit, value, back) = match step.as_str() {
        "production" => (
            1,
            "ฤดูกาลนี้คาดว่าจะขายทุเรียนได้กี่กิโลกรัม",
            "ผลผลิตที่ขายได้โดยประมาณ",
            "ใช้ยอดที่คาดว่าจะขายได้จริงหลังหักผลเสียและผลที่ไม่ได้ขาย",
            "กก.",
            record.quick_estimate.sellable_yield_kg,
            format!("/plans/{id}"),
        ),
        "price" => (
            2,
            "คาดว่าจะขายได้ราคาเฉลี่ยกี่บาทต่อกิโลกรัม",
            "ราคาขายเฉลี่ยโดยประมาณ",
            "ถ้ามีหลายเกรด ให้ใช้ราคาเฉลี่ยคร่าว ๆ ของทั้งฤดูกาล",
            "บาท/กก.",
            record.quick_estimate.average_price_per_kg,
            format!("/plans/{id}/quick/production"),
        ),
        _ => (
            3,
            "คาดว่าฤดูกาลนี้มีต้นทุนรวมประมาณเท่าไร",
            "ต้นทุนรวมโดยประมาณ",
            "รวมค่าใช้จ่ายทั้งหมดแบบคร่าว ๆ ก่อน รายละเอียดแยกทีหลังได้",
            "บาท",
            record.quick_estimate.total_cost,
            format!("/plans/{id}/quick/price"),
        ),
    };
    let value = value
        .map(|value| value.normalize().to_string())
        .unwrap_or_default();
    let season = format!(
        "{} · {}",
        season_year_label(record.season_year),
        record.form.name
    );

    view! {
        <section class="page-stack quick-question-page">
            <header class="page-heading compact-heading">
                <div>
                    <p class="eyebrow">{season}</p>
                    <p class="step-count">{format!("ขั้น {position} จาก 3")}</p>
                    <h1>{question}</h1>
                </div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="พักและกลับหน้าฤดูกาล">"×"</A>
            </header>
            <section class="card quick-question-card">
                <ActionForm action=save>
                    <input type="hidden" name="id" value=id/>
                    <input type="hidden" name="step" value=step/>
                    <label>
                        <span>{field_label}</span>
                        <span class="input-with-unit">
                            <input id="quick-value" type="text" name="value" inputmode="decimal" prop:value=value required autofocus aria-describedby="quick-value-help"/>
                            <span class="unit">{unit}</span>
                        </span>
                        <small id="quick-value-help" class="field-hint">{hint}</small>
                    </label>
                    <button class="primary" type="submit">{if position == 3 { "ดูผลประมาณการ" } else { "ถัดไป" }}</button>
                </ActionForm>
                <ServerErrorMessage action=save/>
            </section>
            <A attr:class="button secondary quick-back" href=back>"‹ ย้อนกลับ"</A>
        </section>
    }
}

#[component]
pub fn QuickResultView(record: PlanRecord) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
    let id = record.id;
    let estimate = record.quick_estimate;
    let analysis = calc::analyze_quick(&estimate);
    let resume = quick_resume_path(id, &estimate);
    let complete = analysis.input_issues.is_empty();
    let season = format!(
        "{} · {}",
        season_year_label(record.season_year),
        record.form.name
    );
    let result = if complete {
        let profit = analysis
            .net_profit
            .expect("complete quick result has profit");
        let (profit_label, profit_class) = if profit >= rust_decimal::Decimal::ZERO {
            ("กำไรโดยประมาณ", "hero-value good-text")
        } else {
            ("ขาดทุนโดยประมาณ", "hero-value bad-text")
        };
        view! {
            <section class="card quick-result-card">
                <p class="eyebrow">"ประมาณการเร็ว"</p>
                <p class="result-label">{profit_label}</p>
                <strong class=profit_class>{format!("{} บาท", money(profit.abs()))}</strong>
                <div class="quick-figure-list">
                    <div><span>"รายได้โดยประมาณ"</span><strong>{format!("{} บาท", money(analysis.revenue.expect("complete revenue")))}</strong></div>
                    <div><span>"ต้นทุนต่อกิโลกรัม"</span><strong>{format!("{} บาท/กก.", money(analysis.cost_per_kg.expect("complete cost per kg")))}</strong></div>
                    <div><span>"ราคาขายคุ้มทุน"</span><strong>{format!("{} บาท/กก.", money(analysis.break_even_price_per_kg.expect("complete break-even price")))}</strong></div>
                </div>
            </section>
            <section class="card quick-source-card">
                <h2>"คำนวณจาก 3 ค่านี้"</h2>
                <dl class="season-metadata">
                    <div><dt>"ผลผลิตที่ขายได้"</dt><dd>{format!("{} กก.", money(estimate.sellable_yield_kg.expect("complete yield")))}</dd></div>
                    <div><dt>"ราคาขายเฉลี่ย"</dt><dd>{format!("{} บาท/กก.", money(estimate.average_price_per_kg.expect("complete price")))}</dd></div>
                    <div><dt>"ต้นทุนรวม"</dt><dd>{format!("{} บาท", money(estimate.total_cost.expect("complete total cost")))}</dd></div>
                </dl>
                <p class="caption">"ผลนี้ใช้ประมาณการรวม ยังไม่ใช้เกรด รายการต้นทุน ROI ภาษี หรือคะแนนสุขภาพสวน"</p>
            </section>
        }.into_any()
    } else {
        view! {
            <section class="card empty-state">
                <p class="eyebrow">"ประมาณการเร็ว"</p>
                <h1>"ตอบให้ครบ 3 ข้อก่อนดูผล"</h1>
                <p>"ระบบจะไม่เติมค่าที่ขาดหรือแสดงผลลัพธ์บางส่วนแทนข้อมูลจริง"</p>
                <A attr:class="button primary" href=resume>"ทำประมาณการต่อ"</A>
            </section>
        }
        .into_any()
    };

    view! {
        <section class="page-stack quick-result-page">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">{season}</p><h1>"ผลประมาณการ"</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A>
            </header>
            <Show when=move || record.closed>
                <div class="closed-banner" role="status">"ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"</div>
            </Show>
            {result}
            <Show when=move || complete && !record.closed>
                <section class="card quick-result-actions">
                    <h2>"ทำต่ออย่างไร"</h2>
                    <A attr:class="button secondary" href=format!("/plans/{id}/quick/production")>"แก้ประมาณการ"</A>
                    <ActionForm action=switch>
                        <input type="hidden" name="id" value=id/>
                        <input type="hidden" name="mode" value="detailed"/>
                        <button class="primary" type="submit">"เปลี่ยนเป็นแผนละเอียด"</button>
                    </ActionForm>
                    <ServerErrorMessage action=switch/>
                </section>
            </Show>
            <BottomNav plan_id=id active=NavSection::Home/>
        </section>
    }
}

#[component]
pub fn QuickModeActiveNotice(record: PlanRecord) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
    let id = record.id;
    view! {
        <section class="page-stack">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">{season_year_label(record.season_year)}</p><h1>"ประมาณการเร็วกำลังใช้งาน"</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A>
            </header>
            <section class="card">
                <p>"ผลของฤดูกาลนี้คำนวณจากประมาณการเร็ว ระบบจึงไม่สลับไปใช้ข้อมูลละเอียดโดยอัตโนมัติ"</p>
                <A attr:class="button primary" href=quick_resume_path(id, &record.quick_estimate)>"กลับไปประมาณการเร็ว"</A>
                <Show when=move || !record.closed>
                    <ActionForm action=switch>
                        <input type="hidden" name="id" value=id/>
                        <input type="hidden" name="mode" value="detailed"/>
                        <button class="text-button" type="submit">"เปลี่ยนเป็นแผนละเอียด"</button>
                    </ActionForm>
                </Show>
                <ServerErrorMessage action=switch/>
            </section>
        </section>
    }
}

#[component]
pub fn DetailedModeActiveNotice(record: PlanRecord) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
    let id = record.id;
    view! {
        <section class="page-stack">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">{season_year_label(record.season_year)}</p><h1>"แผนละเอียดกำลังใช้งาน"</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A>
            </header>
            <section class="card">
                <p>"ผลของฤดูกาลนี้คำนวณจากข้อมูลแผนละเอียด ค่าประมาณการเร็วเดิมยังเก็บไว้ แต่ระบบจะไม่สลับกลับไปใช้เอง"</p>
                <A attr:class="button primary" href=format!("/plans/{id}")>"กลับไปดูและแก้แผนละเอียด"</A>
                <Show when=move || !record.closed>
                    <ActionForm action=switch>
                        <input type="hidden" name="id" value=id/>
                        <input type="hidden" name="mode" value="quick"/>
                        <button class="text-button" type="submit">"เปลี่ยนเป็นประมาณการเร็ว"</button>
                    </ActionForm>
                </Show>
                <ServerErrorMessage action=switch/>
            </section>
        </section>
    }
}

#[component]
pub fn PlanSectionRoute() -> impl IntoView {
    let params = use_params_map();
    let key = move || {
        params.with(|params| {
            let id = params.get("id").and_then(|id| id.parse::<i64>().ok());
            let section = params.get("section").unwrap_or_default();
            (id, section)
        })
    };
    let record = Resource::new(key, |(id, _)| async move {
        match id {
            Some(id) => crate::plans::load_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านข้อมูล…"</p> }>
            {move || record.get().map(|result| {
                let (_, section) = key();
                match result {
                    Ok(Some(record)) if section == "dashboard" && record.forecast_mode == calc::ForecastMode::Quick =>
                        view! { <QuickResultView record/> }.into_any(),
                    Ok(Some(record)) if section == "analysis" && record.forecast_mode == calc::ForecastMode::Quick =>
                        view! { <QuickResultView record/> }.into_any(),
                    Ok(Some(record)) if section == "dashboard" =>
                        view! { <PlanDashboardView record/> }.into_any(),
                    Ok(Some(record)) if section == "analysis" =>
                        view! { <PlanAnalysisView record/> }.into_any(),
                    Ok(Some(record)) if EDITABLE_SECTIONS.iter().any(|(slug, _, _)| *slug == section) =>
                        view! { <PlanSectionView record section/> }.into_any(),
                    _ => view! { <section class="card"><h1>"ไม่พบส่วนนี้"</h1><A href="/plans">"กลับไปฤดูกาลของฉัน"</A></section> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn PlanSectionView(record: PlanRecord, section: String) -> impl IntoView {
    if record.forecast_mode == calc::ForecastMode::Quick {
        return view! { <QuickModeActiveNotice record/> }.into_any();
    }
    let form = RwSignal::new(record.form);
    let save = ServerAction::<SavePlan>::new();
    let id = record.id;
    let closed = record.closed;
    let assets = StoredValue::new(record.asset_allocations);
    let starting_capital = record.starting_capital;
    let season_label = format!(
        "{} · {}",
        season_year_label(record.season_year),
        form.get().name
    );
    let title = EDITABLE_SECTIONS
        .iter()
        .find(|(slug, _, _)| *slug == section)
        .map(|(_, title, _)| *title)
        .unwrap_or("ข้อมูลแผน");
    let section_for_fields = section.clone();
    let section_for_validation = section.clone();
    let section_for_button = StoredValue::new(section.clone());

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">{season_label}</p><h1>{title}</h1></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A>
            </header>
            <Show when=move || closed>
                <div class="closed-banner" role="status">"ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"</div>
            </Show>
            <ActionForm action=save>
                <input type="hidden" name="id" value=id/>
                <input type="hidden" name="section" value=section/>
                <input type="hidden" name="form_json" value=move || serde_json::to_string(&form.get()).unwrap_or_default()/>
                <SectionFields section=section_for_fields form closed plan_id=id assets/>
                <ValidationSummary form section=section_for_validation/>
                <Show when=move || !closed>
                    <button class="primary save-button" type="submit" disabled=move || section_for_button.with_value(|section| !form.get().section_errors(section).is_empty())>"บันทึกส่วนนี้"</button>
                </Show>
            </ActionForm>
            <ServerMessage action=save/>
            <LiveTotal form assets starting_capital/>
            <BottomNav plan_id=id active=NavSection::Input/>
        </section>
    }.into_any()
}

#[component]
fn SectionFields(
    section: String,
    form: RwSignal<PlanForm>,
    closed: bool,
    plan_id: i64,
    assets: StoredValue<Vec<calc::AssetAllocation>>,
) -> impl IntoView {
    match section.as_str() {
        "market" => view! { <MarketFields form closed/> }.into_any(),
        "production" => view! { <ProductionFields form closed/> }.into_any(),
        "expenses" => view! { <ExpenseFields form closed/> }.into_any(),
        "variable-costs" => view! { <VariableCostFields form closed/> }.into_any(),
        "fixed-costs" => view! { <FixedCostFields form closed plan_id assets/> }.into_any(),
        "targets" => view! { <TargetFields form closed/> }.into_any(),
        "health" => view! { <HealthFields form closed/> }.into_any(),
        _ => view! { <p>"ไม่พบข้อมูลส่วนนี้"</p> }.into_any(),
    }
}

#[component]
fn MarketFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <p class="section-intro">"ทุกข้อในหน้านี้ไม่บังคับ ผลกำไรหลักคำนวณได้โดยไม่ต้องกรอกส่วนนี้ กรอกเมื่ออยากเทียบกิโลที่คาดว่าจะขายได้กับยอดที่ผู้ซื้อคุยไว้ หรืออยากจดบริบทของปีนี้ไว้"</p>
        <PlanField field_id="market-target-customer" label="ปีนี้จะขายให้ใคร" formal_term="ลูกค้าเป้าหมาย" hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ เก็บไว้เป็นบริบทวางแผน" example="เช่น ล้งส่งออก ตลาดค้าส่ง ขายหน้าสวน" value=Signal::derive(move || form.get().market.target_customer) on_value=Callback::new(move |v| form.update(|f| f.market.target_customer = v)) closed/>
        <PlanField field_id="market-buyer-committed-kg" label="มีใครบอกว่าจะรับกี่กิโล" formal_term="ยอดรับซื้อที่คาดไว้" unit="กก." numeric=true hint="ไม่บังคับ · ใช้ในการคำนวณส่วนต่างและสัดส่วนที่ส่งได้ เทียบกับกิโลที่คาดว่าจะขายได้" example="เช่น ล้งบอกว่าจะรับ 25,000" outcome="ถ้าเว้นว่าง ผลกำไรยังคำนวณได้ แต่หน้าวิเคราะห์จะไม่แสดงการเทียบกับผู้ซื้อ" value=Signal::derive(move || form.get().market.buyer_committed_kg) on_value=Callback::new(move |v| form.update(|f| f.market.buyer_committed_kg = v)) closed/>
        <PlanField field_id="market-minimum-price" label="ราคาต่ำสุดที่ยอมขาย" formal_term="ราคาขั้นต่ำที่รับได้" unit="บาท/กก." numeric=true hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ เก็บไว้เทียบกับราคาที่กรอกในหน้าผลผลิตด้วยตาตัวเอง" example="เช่น ต่ำกว่า 60 ไม่ขาย" value=Signal::derive(move || form.get().market.minimum_price_per_kg) on_value=Callback::new(move |v| form.update(|f| f.market.minimum_price_per_kg = v)) closed/>
        <PlanField field_id="market-sales-period" label="จะขายช่วงไหน" formal_term="ช่วงเวลาขาย" hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ เก็บไว้เป็นบริบทวางแผน" example="เช่น พฤษภาคม–มิถุนายน" value=Signal::derive(move || form.get().market.sales_period) on_value=Callback::new(move |v| form.update(|f| f.market.sales_period = v)) closed/>
        <PlanField field_id="market-sales-channels" label="ขายกี่ทาง" formal_term="จำนวนช่องทางขาย" unit="ช่องทาง" numeric=true hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ ใช้ทบทวนตอนประเมินความพร้อมของสวน" example="เช่น ส่งล้ง 1 ทาง กับขายเองอีก 1 ทาง = 2" value=Signal::derive(move || form.get().market.sales_channels) on_value=Callback::new(move |v| form.update(|f| f.market.sales_channels = v)) closed/>
        <PlanField field_id="market-largest-buyer" label="รายใหญ่สุดรับกี่เปอร์เซ็นต์ของทั้งหมด" formal_term="การกระจุกตัวของลูกค้า" unit="%" numeric=true hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ ใช้ทบทวนว่าพึ่งผู้ซื้อรายเดียวมากแค่ไหน" example="เช่น ล้งเจ้าเดียวรับ 70" value=Signal::derive(move || form.get().market.largest_buyer_percent) on_value=Callback::new(move |v| form.update(|f| f.market.largest_buyer_percent = v)) closed/>
        <PlanField field_id="market-quality" label="ผู้ซื้อต้องการคุณภาพแบบไหน" formal_term="ข้อกำหนดคุณภาพ" hint="ไม่บังคับ · ไม่ใช้ในการคำนวณ เก็บไว้เป็นบริบทวางแผน" example="เช่น น้ำหนักต่อลูก ความสุก เกรดที่รับ" value=Signal::derive(move || form.get().market.quality_requirements) on_value=Callback::new(move |v| form.update(|f| f.market.quality_requirements = v)) closed/>
    </section> }
}

#[component]
fn BranchChoice(
    legend: &'static str,
    name: &'static str,
    options: [(&'static str, &'static str, &'static str); 2],
    selected: Signal<&'static str>,
    on_select: Callback<String>,
    closed: bool,
) -> impl IntoView {
    view! { <fieldset class="branch-choice" disabled=closed>
        <legend>{legend}</legend>
        {options.into_iter().map(|(value, label, description)| view! {
            <label class="branch-option">
                <input type="radio" name=name value=value prop:checked=move || selected.get() == value on:change=move |_| on_select.run(value.to_owned())/>
                <span><strong>{label}</strong><small>{description}</small></span>
            </label>
        }).collect_view()}
    </fieldset> }
}

#[component]
fn ProductionFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    // Repeat rows re-render only when a row is added or removed. Re-rendering
    // on every keystroke recreates the focused input and drops the keystroke.
    let grade_rows = Memo::new(move |_| form.with(|f| f.grades.len()));
    let yield_source = Signal::derive(move || match form.get().production.yield_source {
        YieldSource::Direct => "direct",
        YieldSource::Derived => "derived",
    });
    let price_source = Signal::derive(move || match form.get().production.price_source {
        PriceSource::Average => "average",
        PriceSource::ByGrade => "by_grade",
    });
    let grade_entry = Signal::derive(move || match form.get().grade_entry {
        GradeEntry::Percent => "percent",
        GradeEntry::Kilograms => "kilograms",
    });
    let derived_note = move || {
        form.get().derived_sellable_yield_kg().map(|kg| {
            format!(
                "จากข้อมูลต้นทุเรียนที่กรอกไว้ คำนวณได้ประมาณ {} กก. ตัวเลขนี้ไม่ถูกนำไปใช้ จนกว่าจะเลือกคำนวณจากต้น",
                money(kg)
            )
        })
    };
    let derived_result = move || {
        form.get()
            .derived_sellable_yield_kg()
            .map(|kg| format!("คำนวณได้ประมาณ {} กก. หลังหักส่วนที่เสีย", money(kg)))
            .unwrap_or_else(|| "ยังขาดจำนวนต้น ลูกต่อต้น น้ำหนักต่อลูก หรือส่วนที่เสีย จึงยังคำนวณกิโลไม่ได้".into())
    };
    let weighted_note = move || {
        form.get().weighted_grade_price_per_kg().map(|price| {
            format!(
                "จากเกรดที่กรอกไว้ ถ่วงน้ำหนักได้ประมาณ {} บาท/กก. ตัวเลขนี้ไม่ถูกนำไปใช้ จนกว่าจะเลือกแยกตามเกรด",
                money(price)
            )
        })
    };
    let kg_entry_available = move || form.get().sellable_yield_kg().is_some();
    let grade_total = move || {
        let form = form.get();
        match form.grade_entry {
            GradeEntry::Percent => (
                form.grade_total_percent() == Some(rust_decimal::Decimal::ONE_HUNDRED),
                form.grade_total_percent()
                    .map(|v| format!("รวม {}%", v.normalize()))
                    .unwrap_or_else(|| "กรอกสัดส่วน".into()),
            ),
            GradeEntry::Kilograms => {
                let total = form.grade_total_kg();
                let sellable = form.sellable_yield_kg();
                (
                    total.is_some() && total == sellable,
                    match (total, sellable) {
                        (Some(total), Some(sellable)) => {
                            format!("รวม {} จาก {} กก.", money(total), money(sellable))
                        }
                        _ => "กรอกกิโลของแต่ละเกรด".into(),
                    },
                )
            }
        }
    };
    view! { <div class="field-stack">
        <section class="card field-stack">
            <div class="section-title"><div><h2>"คาดว่าจะขายได้กี่กิโล"</h2><small class="formal-term">"ผลผลิตขายได้"</small></div></div>
            <BranchChoice legend="ตอบแบบไหนสะดวกกว่า" name="yield-source" options=[("direct", "รู้ตัวเลขรวมแล้ว", "กรอกกิโลที่คาดว่าจะขายได้ทั้งฤดู"), ("derived", "คำนวณจากต้นทุเรียน", "นับต้น ลูกต่อต้น น้ำหนัก แล้วหักส่วนที่เสีย")] selected=yield_source on_select=Callback::new(move |v: String| form.update(|f| f.production.yield_source = if v == "direct" { YieldSource::Direct } else { YieldSource::Derived })) closed/>
            <Show when=move || yield_source.get() == "direct">
                <PlanField field_id="production-sellable-yield" label="กิโลที่คาดว่าจะขายได้ทั้งฤดู" formal_term="ผลผลิตขายได้" unit="กก." numeric=true hint="นับเฉพาะที่ขายได้จริง ไม่รวมลูกที่เสียหรือตกไซซ์ ถ้ายังไม่รู้ ให้เว้นว่างไว้ก่อน ระบบจะบอกว่าผลไหนยังคำนวณไม่ได้ และไม่เดาตัวเลขให้" example="เช่น ปีก่อนขายได้ 19,950" outcome="ใช้คูณกับราคาขาย ได้เป็นรายได้โดยประมาณ" value=Signal::derive(move || form.get().production.sellable_yield_kg) on_value=Callback::new(move |v| form.update(|f| f.production.sellable_yield_kg = v)) closed/>
                {move || derived_note().map(|note| view! { <p class="branch-note" aria-live="polite">{note}</p> })}
            </Show>
            <Show when=move || yield_source.get() == "derived">
                <PlanField field_id="production-trees" label="มีต้นที่ให้ลูกกี่ต้น" formal_term="ต้นที่ให้ผลผลิต" unit="ต้น" numeric=true hint="นับเฉพาะต้นที่คาดว่าจะเก็บได้ปีนี้" example="เช่น 200" value=Signal::derive(move || form.get().production.producing_trees) on_value=Callback::new(move |v| form.update(|f| f.production.producing_trees = v)) closed/>
                <PlanField field_id="production-fruits-per-tree" label="ต้นหนึ่งได้กี่ลูก" formal_term="ผลต่อต้น" unit="ลูก" numeric=true hint="ค่าเฉลี่ยทั้งสวน" example="เช่น 35" value=Signal::derive(move || form.get().production.fruits_per_tree) on_value=Callback::new(move |v| form.update(|f| f.production.fruits_per_tree = v)) closed/>
                <PlanField field_id="production-fruit-weight" label="ลูกหนึ่งหนักกี่กิโล" formal_term="น้ำหนักเฉลี่ยต่อผล" unit="กก." numeric=true hint="ค่าเฉลี่ยต่อลูก" example="เช่น 3" value=Signal::derive(move || form.get().production.average_fruit_weight_kg) on_value=Callback::new(move |v| form.update(|f| f.production.average_fruit_weight_kg = v)) closed/>
                <PlanField field_id="production-loss" label="เสียไปกี่เปอร์เซ็นต์ก่อนถึงมือผู้ซื้อ" formal_term="สัดส่วนสูญเสีย" unit="%" numeric=true hint="ลูกร่วง หนอน ตกไซซ์ หรือเสียระหว่างทาง" example="เช่น 5" outcome="กิโลที่คาดว่าจะขายได้ = ต้น × ลูกต่อต้น × น้ำหนัก แล้วหักส่วนนี้ออก" value=Signal::derive(move || form.get().production.loss_percent) on_value=Callback::new(move |v| form.update(|f| f.production.loss_percent = v)) closed/>
                <p class="branch-note" aria-live="polite">{derived_result}</p>
            </Show>
            <PlanField field_id="production-area" label="สวนกี่ไร่" formal_term="พื้นที่ให้ผลผลิต" unit="ไร่" numeric=true hint="ไม่บังคับ · ใช้เฉพาะตัวเลขต่อไร่ในหน้าวิเคราะห์ ไม่กระทบกำไร" example="เช่น 10" value=Signal::derive(move || form.get().production.area_rai) on_value=Callback::new(move |v| form.update(|f| f.production.area_rai = v)) closed/>
        </section>
        <section class="card field-stack">
            <div class="section-title"><div><h2>"แต่ละแบบขายราคาเท่าไร"</h2><small class="formal-term">"ราคาขายเฉลี่ยถ่วงน้ำหนัก"</small></div></div>
            <BranchChoice legend="ตอบแบบไหนสะดวกกว่า" name="price-source" options=[("average", "ราคาเดียวทั้งหมด", "คิดเฉลี่ยทุกลูกเป็นราคาเดียว"), ("by_grade", "แยกตามเกรด", "แต่ละเกรดขายได้ราคาต่างกัน")] selected=price_source on_select=Callback::new(move |v: String| form.update(|f| f.production.price_source = if v == "average" { PriceSource::Average } else { PriceSource::ByGrade })) closed/>
            <Show when=move || price_source.get() == "average">
                <PlanField field_id="production-average-price" label="ขายได้กิโลละเท่าไร" formal_term="ราคาขายเฉลี่ย" unit="บาท/กก." numeric=true hint="ราคาเฉลี่ยที่คาดว่าจะได้รับทั้งฤดู ถ้ายังไม่รู้ ให้เว้นว่างไว้ก่อน" example="เช่น 82.5" outcome="ใช้คูณกับกิโลที่คาดว่าจะขายได้ ได้เป็นรายได้โดยประมาณ" value=Signal::derive(move || form.get().production.average_price_per_kg) on_value=Callback::new(move |v| form.update(|f| f.production.average_price_per_kg = v)) closed/>
                {move || weighted_note().map(|note| view! { <p class="branch-note" aria-live="polite">{note}</p> })}
                <Show when=move || !form.get().grades.is_empty()><p class="caption">"เกรดที่เคยกรอกไว้ยังเก็บอยู่ กลับมาใช้ได้เมื่อเลือกแยกตามเกรด"</p></Show>
            </Show>
            <Show when=move || price_source.get() == "by_grade">
                <div class="section-title"><div><h3>"แต่ละเกรดมีสัดส่วนเท่าไร"</h3><small class="formal-term">"สัดส่วนเกรด"</small><p>"เพิ่มหรือลดได้สูงสุด 10 เกรด"</p></div><strong class=move || if grade_total().0 { "good-text" } else { "bad-text" }>{move || grade_total().1}</strong></div>
                <fieldset class="branch-choice compact" disabled=closed>
                    <legend>"กรอกสัดส่วนเป็น"</legend>
                    <label class="branch-option"><input type="radio" name="grade-entry" value="percent" prop:checked=move || grade_entry.get() == "percent" on:change=move |_| form.update(|f| { f.set_grade_entry(GradeEntry::Percent); })/><span><strong>"เปอร์เซ็นต์"</strong></span></label>
                    <label class="branch-option"><input type="radio" name="grade-entry" value="kilograms" prop:checked=move || grade_entry.get() == "kilograms" disabled=move || !kg_entry_available() on:change=move |_| form.update(|f| { f.set_grade_entry(GradeEntry::Kilograms); })/><span><strong>"กิโลกรัม"</strong></span></label>
                </fieldset>
                <Show when=move || !kg_entry_available()><p class="branch-note" aria-live="polite">"กรอกเป็นกิโลกรัมได้เมื่อรู้กิโลที่คาดว่าจะขายได้แล้ว ตอนนี้จึงกรอกได้เฉพาะเปอร์เซ็นต์"</p></Show>
                {move || (0..grade_rows.get()).map(|index| view! {
                    <div class="repeat-row">
                        <PlanField label="ชื่อเกรด" example="เช่น A, B, C, ตกเกรด" value=Signal::derive(move || form.get().grades.get(index).map(|g| g.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.name = v })) closed/>
                        <Show when=move || grade_entry.get() == "percent">
                            <PlanField label="เกรดนี้กี่เปอร์เซ็นต์ของทั้งหมด" unit="%" numeric=true value=Signal::derive(move || form.get().grades.get(index).map(|g| g.share_percent.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.share_percent = v })) closed/>
                            {move || form.get().grade_kg_from_percent(index).map(|kg| view! { <p class="conversion">{format!("≈ {} กก.", money(kg))}</p> })}
                        </Show>
                        <Show when=move || grade_entry.get() == "kilograms">
                            <PlanField label="เกรดนี้กี่กิโล" unit="กก." numeric=true value=Signal::derive(move || form.get().grades.get(index).map(|g| g.share_kg.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.share_kg = v })) closed/>
                            {move || form.get().grade_percent_from_kg(index).map(|p| view! { <p class="conversion">{format!("≈ {}% ของกิโลที่คาดว่าจะขายได้", money(p))}</p> })}
                        </Show>
                        <PlanField label="เกรดนี้ขายได้กิโลละเท่าไร" unit="บาท/กก." numeric=true value=Signal::derive(move || form.get().grades.get(index).map(|g| g.price_per_kg.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.price_per_kg = v })) closed/>
                        <label class="check-field"><input type="checkbox" prop:checked=move || form.with(|f| f.grades.get(index).is_some_and(|g| g.counts_as_quality_grade)) disabled=closed on:change=move |event| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.counts_as_quality_grade = event_target_checked(&event) })/><span>"นับเป็นเกรดคุณภาพ"</span></label>
                        <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.grades.len() { f.grades.remove(index); } })>"ลบเกรดนี้"</button></Show>
                    </div>
                }).collect_view()}
                <Show when=move || !closed && form.get().grades.len() < 10><button class="secondary" type="button" on:click=move |_| form.update(|f| f.grades.push(GradeForm::default()))>"+ เพิ่มเกรด"</button></Show>
                {move || form.get().weighted_grade_price_per_kg().map(|price| view! { <p class="branch-note" aria-live="polite">{format!("ราคาเฉลี่ยถ่วงน้ำหนักจากทุกเกรด ประมาณ {} บาท/กก.", money(price))}</p> })}
            </Show>
        </section>
    </div> }
}

#[component]
fn SectionStateChoice(
    form: RwSignal<PlanForm>,
    fixed: bool,
    closed: bool,
    #[prop(optional)] asset_depreciation: rust_decimal::Decimal,
) -> impl IntoView {
    let asset_depreciation = (!asset_depreciation.is_zero()).then_some(asset_depreciation);
    let selected = Signal::derive(move || {
        let form = form.get();
        let state = if fixed {
            form.fixed_cost_state
        } else {
            form.variable_cost_state
        };
        match state {
            CostSectionState::ConfirmedNone => "confirmed_none",
            _ => "unknown",
        }
    });
    let legend = match (fixed, asset_depreciation) {
        (true, Some(_)) => "นอกจากค่าเสื่อมของที่เลือกไว้ ปีนี้ยังมีอะไรต้องจ่ายแม้ไม่มีทุเรียนขายไหม",
        (true, None) => "แม้ไม่มีทุเรียนขาย ปีนี้ยังมีอะไรต้องจ่ายไหม",
        (false, _) => "ปีนี้มีค่าใช้จ่ายที่เพิ่มตามการผลิตไหม",
    };
    let none_description = match (fixed, asset_depreciation) {
        (true, Some(_)) => "ไม่มีค่าใช้จ่ายประจำอื่น ระบบจะนับเฉพาะค่าเสื่อมของที่เลือกไว้",
        (true, None) => "ปีนี้ไม่มีอะไรที่ต้องจ่ายแม้ไม่มีทุเรียนขาย ระบบจะนับเป็นศูนย์",
        (false, _) => "ปีนี้ไม่มีค่าใช้จ่ายที่เพิ่มตามการผลิต ระบบจะนับเป็นศูนย์",
    };
    view! {
        <BranchChoice legend=legend name=if fixed { "fixed-cost-state" } else { "variable-cost-state" } options=[("unknown", "ยังไม่รู้ / ข้ามก่อน", "ผลที่ต้องใช้ตัวเลขนี้จะยังไม่ขึ้น จนกว่าจะกรอกหรือยืนยัน"), ("confirmed_none", "ยืนยันว่าไม่มี", none_description)] selected=selected on_select=Callback::new(move |v: String| form.update(|f| {
            let state = if v == "confirmed_none" { CostSectionState::ConfirmedNone } else { CostSectionState::Unknown };
            if fixed { f.fixed_cost_state = state } else { f.variable_cost_state = state }
        })) closed/>
        <p class="caption">"ถ้ามีค่าใช้จ่ายจริง กดเพิ่มรายการด้านล่างได้เลย คำตอบข้อนี้จะเปลี่ยนเป็นมีรายการโดยอัตโนมัติ"</p>
    }
}

#[component]
fn VariableCostFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    let variable_rows = Memo::new(move |_| form.with(|f| f.variable_costs.len()));
    let sellable = move || form.get().sellable_yield_kg();
    view! { <section class="card field-stack">
        <div class="section-title"><div><h2>"ค่าใช้จ่ายที่เพิ่มเมื่อทำหรือขายมากขึ้น"</h2><small class="formal-term">"ต้นทุนผันแปร"</small><p>"เช่น ปุ๋ย ยา น้ำ ไฟ น้ำมัน คนดูแล คนเก็บ ขนส่ง กล่อง ถ้าจำได้แต่ไม่รู้ว่าเป็นแบบไหน จดไว้ที่ค่าใช้จ่ายที่จำได้ก่อนได้"</p></div></div>
        <Show when=move || form.get().variable_costs.is_empty()><SectionStateChoice form fixed=false closed/></Show>
        {move || (0..variable_rows.get()).map(|index| {
            let kind = form.with_untracked(|f| f.variable_costs.get(index).map_or(VariableCostKind::Other, |l| l.kind));
            let total_only = Signal::derive(move || if form.get().variable_costs.get(index).is_some_and(|l| l.total_only) { "total" } else { "per_unit" });
            let default_note = move || {
                let form = form.get();
                let line = form.variable_costs.get(index)?;
                if line.total_only || !line.kind.follows_sellable_yield() || !line.quantity.trim().is_empty() {
                    return None;
                }
                Some(match sellable() {
                    Some(kg) => format!("เว้นจำนวนไว้ ระบบใช้กิโลที่คาดว่าจะขายได้ {} กก. เป็นจำนวนของรายการนี้", money(kg)),
                    None => "เว้นจำนวนไว้ ระบบจะใช้กิโลที่คาดว่าจะขายได้เป็นจำนวน แต่ตอนนี้ยังไม่รู้กิโลนั้น รายการนี้จึงยังคำนวณไม่ได้".into(),
                })
            };
            view! {
            <div class="repeat-row">
                <PlanField label="ค่าอะไร" example="เช่น ปุ๋ยรอบแรก ค่าจ้างคนเก็บ" value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.name = v })) closed/>
                <label><span>"เป็นค่าใช้จ่ายกลุ่มไหน"</span><small class="formal-term">"ประเภทต้นทุนผันแปร"</small>{if closed { view! { <p class="readonly-value">{variable_kind_label(kind)}</p> }.into_any() } else { view! { <select on:change=move |event| { let kind = parse_variable_kind(&event_target_value(&event)); form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.kind = kind }); }>{variable_kind_options(kind)}</select> }.into_any() }}</label>
                <BranchChoice legend="รู้ตัวเลขแบบไหน" name="" options=[("per_unit", "รู้จำนวนกับราคาต่อหน่วย", "เช่น ปุ๋ย 5,000 กก. กก.ละ 20"), ("total", "รู้แค่ยอดรวม", "เช่น จ่ายไปทั้งหมด 12,000 บาท")] selected=total_only on_select=Callback::new(move |v: String| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.total_only = v == "total" })) closed/>
                <Show when=move || total_only.get() == "per_unit">
                    <PlanField label="จำนวน" numeric=true value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.quantity.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.quantity = v })) closed/>
                    {move || default_note().map(|note| view! { <p class="branch-note quantity-default" aria-live="polite">{note}</p> })}
                    <PlanField label="หน่วย" example="เช่น กก. ลิตร วัน เที่ยว" value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.unit.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.unit = v })) closed/>
                    <PlanField label="ราคาต่อหน่วย" unit="บาท" numeric=true value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.unit_price.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.unit_price = v })) closed/>
                </Show>
                <Show when=move || total_only.get() == "total">
                    <PlanField label="ยอดรวมทั้งฤดู" unit="บาท" numeric=true hint="รายการที่รู้แค่ยอดรวม จะไม่มีตัวเลขผลผลิตต่อหน่วยในหน้าวิเคราะห์" value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.total_amount.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.total_amount = v })) closed/>
                </Show>
                <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.variable_costs.len() { f.variable_costs.remove(index); } })>"ลบรายการ"</button></Show>
            </div>
        }}).collect_view()}
        <Show when=move || !closed><button class="secondary" type="button" on:click=move |_| form.update(|f| f.variable_costs.push(VariableCostForm::default()))>"+ เพิ่มค่าใช้จ่าย"</button></Show>
    </section> }
}

#[component]
fn FixedCostFields(
    form: RwSignal<PlanForm>,
    closed: bool,
    plan_id: i64,
    assets: StoredValue<Vec<calc::AssetAllocation>>,
) -> impl IntoView {
    let fixed_rows = Memo::new(move |_| form.with(|f| f.fixed_costs.len()));
    let asset_depreciation = assets.with_value(|assets| included_asset_depreciation(assets));
    let asset_rows = assets.with_value(|assets| {
        assets
            .iter()
            .map(|asset| (asset.facts.name.clone(), asset.annual_depreciation))
            .collect::<Vec<_>>()
    });
    // The same arithmetic the dashboard uses, so the total here cannot drift
    // from the fixed cost shown there.
    let total = move || {
        form.get().to_plan().ok().and_then(|plan| {
            assets.with_value(|assets| {
                calc::analyze_with_assets(&plan, assets, None)
                    .cost
                    .fixed_cost
            })
        })
    };
    view! { <section class="card field-stack">
        <div class="section-title"><div><h2>"แม้ปีนี้ไม่มีทุเรียนขาย ยังต้องจ่ายอะไรอยู่"</h2><small class="formal-term">"ต้นทุนคงที่"</small><p>"เช่น ค่าเช่าที่ เงินเดือนคนงานประจำ ดอกเบี้ย ค่าเสื่อมของที่ใช้หลายปี"</p></div></div>
        <details class="explanation"><summary>"ⓘ จ่ายเงินจริง กับ เฉลี่ยจากของหลายปี ต่างกันอย่างไร"</summary><h3>"คืออะไร"</h3><p>"ค่าเสื่อมระบบน้ำและค่าเสื่อมรถ เป็นต้นทุนที่ลงบัญชีแต่ปีนี้ไม่ได้ควักเงินจ่าย ส่วนค่าเช่า ดอกเบี้ย และค่าแรงประจำ จ่ายจริงทุกปี"</p><h3>"ใช้ยังไง"</h3><p>"ตอบให้ตรงตอนกรอกแต่ละรายการ"</p><h3>"ทำไมต้องมี"</h3><p>"เป็นสิ่งเดียวที่ทำให้กระแสเงินสดกับกำไรสุทธิต่างกันได้"</p><h3>"ไม่ใส่ได้ไหม"</h3><p>"ตอบผิดได้ แต่เงินสดที่เหลือจะผิดตาม"</p></details>

        <div class="cost-group cost-group-manual">
            <div class="cost-group-title"><h3>"กรอกเอง"</h3><small class="formal-term">"ค่าใช้จ่ายประจำที่บันทึกที่หน้านี้"</small></div>
            <Show when=move || form.get().fixed_costs.is_empty()><SectionStateChoice form fixed=true closed asset_depreciation=asset_depreciation.unwrap_or_default()/></Show>
            {move || (0..fixed_rows.get()).map(|index| {
                let cash = Signal::derive(move || if form.get().fixed_costs.get(index).is_some_and(|l| l.cash_kind == CashKind::NonCash) { "non_cash" } else { "cash" });
                view! {
                <div class="repeat-row">
                    <PlanField label="ค่าอะไร" example="เช่น ค่าเช่าที่ เงินเดือนคนงาน" value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.name = v })) closed/>
                    <BranchChoice legend="ปีนี้ต้องจ่ายเงินจริงไหม" name="" options=[("cash", "จ่ายเงินจริงปีนี้", "ค่าเช่า เงินเดือน ดอกเบี้ย · ต้นทุนเงินสด"), ("non_cash", "เป็นค่าใช้ของหลายปีที่เฉลี่ยลงฤดูนี้", "ค่าเสื่อมระบบน้ำ รถ โรงเรือน · ต้นทุนไม่ใช่เงินสด")] selected=cash on_select=Callback::new(move |v: String| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.cash_kind = if v == "non_cash" { CashKind::NonCash } else { CashKind::Cash } })) closed/>
                    <PlanField label="ปีละเท่าไร" formal_term="จำนวนต่อปี" unit="บาท" numeric=true value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.amount_per_year.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.amount_per_year = v })) closed/>
                    <PlanField label="เงินก้อนที่ลงไปกับรายการนี้ (ถ้ามี)" formal_term="เงินลงทุน" unit="บาท" numeric=true hint="ค่าใช้จ่ายประจำที่ไม่มีเงินก้อนเริ่มต้น เว้นช่องนี้ได้ ของที่ใช้หลายปี บันทึกที่หน้าของที่ใช้หลายปีจะคิดค่าเสื่อมให้เอง" value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.investment_base.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.investment_base = v })) closed/>
                    <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.fixed_costs.len() { f.fixed_costs.remove(index); } })>"ลบรายการ"</button></Show>
                </div>
            }}).collect_view()}
            <Show when=move || !closed><button class="secondary" type="button" on:click=move |_| form.update(|f| f.fixed_costs.push(FixedCostForm::default()))>"+ เพิ่มค่าใช้จ่ายประจำ"</button></Show>
        </div>

        <div class="cost-group cost-group-assets">
            <div class="cost-group-title"><h3>"ของที่ใช้หลายปี · ค่าเสื่อม"</h3><small class="formal-term">"ต้นทุนไม่ใช่เงินสด · เฉลี่ยจากของที่รวมในฤดูนี้"</small></div>
            {if asset_rows.is_empty() {
                view! {
                    <p class="muted">"ยังไม่มีของที่ใช้หลายปีที่รวมในฤดูนี้"</p>
                    <A attr:class="text-button" href=format!("/plans/{plan_id}/assets")>"เลือกของที่ใช้หลายปี"</A>
                }.into_any()
            } else {
                view! {
                    <ul class="asset-cost-list">
                        {asset_rows.into_iter().map(|(name, depreciation)| view! {
                            <li class="asset-cost-row">
                                <span class="asset-cost-name">{name}</span>
                                <span class="asset-cost-amount">{format!("{} บาท/ปี", money(depreciation))}</span>
                                <A attr:class="text-button" href=format!("/plans/{plan_id}/assets")>"ดูที่หน้าของที่ใช้หลายปี"</A>
                            </li>
                        }).collect_view()}
                    </ul>
                    <p class="caption">"ระบบรวมให้เองจากของที่เลือกไว้ ไม่ต้องกรอกซ้ำที่นี่ แก้ไขหรือเอาออกได้ที่หน้าของที่ใช้หลายปี"</p>
                }.into_any()
            }}
        </div>

        <div class="cost-total fixed-cost-total" aria-live="polite">
            <span class="cost-total-label">"รวมค่าใช้จ่ายประจำฤดูนี้"<small class="formal-term">"ต้นทุนคงที่รวม"</small></span>
            <strong class="cost-total-amount">{move || total().map_or_else(|| "ยังไม่รู้".to_string(), |total| format!("{} บาท/ปี", money(total)))}</strong>
            {move || (total().is_none() && asset_depreciation.is_some()).then(|| view! {
                <small class="cost-total-note">{format!("ค่าเสื่อม {} บาท/ปี รวมอยู่แล้ว แต่ยอดรวมยังไม่รู้ จนกว่าจะกรอกหรือยืนยันส่วนที่กรอกเอง", money(asset_depreciation.unwrap_or_default()))}</small>
            })}
        </div>
    </section> }
}

#[component]
fn ExpenseFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    let expense_rows = Memo::new(move |_| form.with(|f| f.unclassified.len()));
    let classifying = RwSignal::new(None::<usize>);
    let grows = RwSignal::new(None::<bool>);
    let cash = RwSignal::new(None::<bool>);
    let kind = RwSignal::new(VariableCostKind::Other);
    let reset = move || {
        classifying.set(None);
        grows.set(None);
        cash.set(None);
        kind.set(VariableCostKind::Other);
    };
    view! { <section class="card field-stack">
        <div class="section-title"><div><h2>"จำค่าใช้จ่ายได้แต่ยังไม่รู้ว่าเป็นแบบไหน"</h2><small class="formal-term">"รายการรอจัดประเภท"</small><p>"จดชื่อกับยอดไว้ก่อน ตัวเลขจะยังไม่ถูกนับ จนกว่าจะตอบว่าเป็นค่าใช้จ่ายแบบไหน"</p></div></div>
        {move || (0..expense_rows.get()).map(|index| view! {
            <div class="repeat-row">
                <PlanField label="ค่าอะไร" example="เช่น จ่ายคนขับรถเดือนสาม" value=Signal::derive(move || form.get().unclassified.get(index).map(|e| e.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(e) = f.unclassified.get_mut(index) { e.name = v })) closed/>
                <PlanField label="เท่าไร" unit="บาท" numeric=true hint="ถ้ายังไม่รู้ยอด เว้นว่างไว้ก่อนได้" value=Signal::derive(move || form.get().unclassified.get(index).map(|e| e.amount.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(e) = f.unclassified.get_mut(index) { e.amount = v })) closed/>
                <PlanField label="โน้ต" example="เช่น ยังไม่แน่ใจว่ารวมค่าน้ำมันหรือยัง" value=Signal::derive(move || form.get().unclassified.get(index).map(|e| e.note.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(e) = f.unclassified.get_mut(index) { e.note = v })) closed/>
                <Show when=move || !closed && classifying.get() != Some(index)>
                    <button class="secondary" type="button" on:click=move |_| { reset(); classifying.set(Some(index)); }>"บอกว่าเป็นแบบไหน"</button>
                    <button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.unclassified.len() { f.unclassified.remove(index); } })>"ลบรายการ"</button>
                </Show>
                <Show when=move || !closed && classifying.get() == Some(index)>
                    <div class="classify-flow">
                        <BranchChoice legend="ถ้าปีนี้ปลูกหรือขายมากขึ้น ค่านี้จะเพิ่มตามไหม" name="classify-grows" options=[("yes", "เพิ่มตาม", "ปุ๋ย ยา คนเก็บ ขนส่ง กล่อง · ต้นทุนผันแปร"), ("no", "จ่ายเท่าเดิมแม้ไม่มีทุเรียนขาย", "ค่าเช่า เงินเดือนประจำ ดอกเบี้ย · ต้นทุนคงที่")] selected=Signal::derive(move || match grows.get() { Some(true) => "yes", Some(false) => "no", None => "" }) on_select=Callback::new(move |v: String| grows.set(Some(v == "yes"))) closed=false/>
                        <Show when=move || grows.get() == Some(true)>
                            <label><span>"ใกล้เคียงกับกลุ่มไหนที่สุด"</span><small class="formal-term">"ประเภทต้นทุนผันแปร"</small><select on:change=move |event| kind.set(parse_variable_kind(&event_target_value(&event)))>{variable_kind_options(VariableCostKind::Other)}</select></label>
                        </Show>
                        <Show when=move || grows.get() == Some(false)>
                            <BranchChoice legend="ปีนี้ต้องจ่ายเงินจริงไหม" name="classify-cash" options=[("yes", "จ่ายเงินจริงปีนี้", "ต้นทุนเงินสด"), ("no", "เป็นค่าใช้ของหลายปีที่เฉลี่ยลงฤดูนี้", "ค่าเสื่อม · ต้นทุนไม่ใช่เงินสด")] selected=Signal::derive(move || match cash.get() { Some(true) => "yes", Some(false) => "no", None => "" }) on_select=Callback::new(move |v: String| cash.set(Some(v == "yes"))) closed=false/>
                            <p class="caption">"ถ้าเป็นเงินก้อนที่ซื้อของใช้หลายปี เช่น ระบบน้ำหรือรถ ให้บันทึกที่หน้าของที่ใช้หลายปีแทน ระบบจะคิดค่าเสื่อมให้เอง"</p>
                        </Show>
                        <button class="primary" type="button" disabled=move || match grows.get() { Some(true) => false, Some(false) => cash.get().is_none(), None => true } on:click=move |_| {
                            let classification = match (grows.get(), cash.get()) {
                                (Some(true), _) => ExpenseClassification::Variable(kind.get()),
                                (Some(false), Some(true)) => ExpenseClassification::Fixed(CashKind::Cash),
                                (Some(false), Some(false)) => ExpenseClassification::Fixed(CashKind::NonCash),
                                _ => return,
                            };
                            form.update(|f| { f.classify_expense(index, classification); });
                            reset();
                        }>"ย้ายไปส่วนที่ถูก"</button>
                        <button class="text-button" type="button" on:click=move |_| reset()>"ยังไม่ตอบตอนนี้"</button>
                    </div>
                </Show>
            </div>
        }).collect_view()}
        <Show when=move || closed && form.get().unclassified.is_empty()><p class="readonly-value">"ไม่มีรายการที่รอบอกประเภท"</p></Show>
        <Show when=move || !closed><button class="secondary" type="button" on:click=move |_| form.update(|f| f.unclassified.push(UnclassifiedExpenseForm::default()))>"+ จดค่าใช้จ่ายที่จำได้"</button></Show>
        <p class="caption">"รายการที่ย้ายแล้วจะไปอยู่ในส่วนต้นทุนผันแปรหรือต้นทุนคงที่ตามคำตอบ กดปุ่มบันทึกด้านล่างเพื่อเก็บทั้งที่จดไว้และที่ย้ายแล้ว"</p>
    </section> }
}

#[component]
fn TargetFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <p class="section-intro">"ส่วนขั้นสูงนี้ไม่บังคับ เป้าหมายเป็นตัวเลขของเจ้าของสวน ระบบจะไม่เดาให้ ถ้าเว้นว่าง หน้าวิเคราะห์จะไม่ตัดสินว่าผ่านหรือไม่ผ่าน"</p>
        <PlanField label="ผลผลิตต่อไร่" unit="กก./ไร่" numeric=true value=Signal::derive(move || form.get().targets.yield_per_rai) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_rai = v)) closed/>
        <PlanField label="ผลผลิตต่อต้น" unit="กก./ต้น" numeric=true value=Signal::derive(move || form.get().targets.yield_per_tree) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_tree = v)) closed/>
        <PlanField label="ผลผลิตต่อวันแรงงาน" unit="กก./วัน" numeric=true value=Signal::derive(move || form.get().targets.yield_per_labor_day) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_labor_day = v)) closed/>
        <PlanField label="ผลผลิตต่อปุ๋ย" unit="กก./กก." numeric=true value=Signal::derive(move || form.get().targets.yield_per_fertilizer_kg) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_fertilizer_kg = v)) closed/>
        <PlanField label="ผลผลิตต่อน้ำ" unit="กก./ลบ.ม." numeric=true value=Signal::derive(move || form.get().targets.yield_per_water_cubic_meter) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_water_cubic_meter = v)) closed/>
        <PlanField label="ผลผลิตต่อไฟฟ้า" unit="กก./kWh" numeric=true value=Signal::derive(move || form.get().targets.yield_per_kwh) on_value=Callback::new(move |v| form.update(|f| f.targets.yield_per_kwh = v)) closed/>
        <PlanField label="สัดส่วนเกรดคุณภาพ" unit="%" numeric=true value=Signal::derive(move || form.get().targets.quality_grade_percent) on_value=Callback::new(move |v| form.update(|f| f.targets.quality_grade_percent = v)) closed/>
        <PlanField label="สัดส่วนสูญเสีย" unit="%" numeric=true value=Signal::derive(move || form.get().targets.loss_percent) on_value=Callback::new(move |v| form.update(|f| f.targets.loss_percent = v)) closed/>
        <PlanField label="ต้นทุนต่อกิโลกรัม" unit="บาท/กก." numeric=true value=Signal::derive(move || form.get().targets.cost_per_kg) on_value=Callback::new(move |v| form.update(|f| f.targets.cost_per_kg = v)) closed/>
    </section> }
}

#[component]
fn HealthFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack health-card">
        <div class="section-title"><div><h2>"สุขภาพสวน 12 ข้อ"</h2><p>"1 = ต้องเร่งปรับปรุง · 5 = แข็งแรง"</p></div><strong>{move || format!("{}/12", form.get().health_scores.iter().filter(|v| !v.is_empty()).count())}</strong></div>
        {HealthQuestion::ALL.into_iter().enumerate().map(|(index, question)| view! {
            <fieldset><legend>{health_question_label(question)}</legend>{if closed {
                view! { <p class="readonly-value">{form.get().health_scores.get(index).cloned().filter(|value| !value.is_empty()).unwrap_or_else(|| "—".into())}</p> }.into_any()
            } else {
                view! { <div class="score-row">{(1_u8..=5).map(|score| view! {
                    <button type="button" class=move || if form.get().health_scores.get(index).is_some_and(|value| value == &score.to_string()) { "score selected" } else { "score" } on:click=move |_| form.update(|f| { if f.health_scores.len() < 12 { f.health_scores.resize(12, String::new()); } f.health_scores[index] = score.to_string(); })>{score}</button>
                }).collect_view()}</div> }.into_any()
            }}</fieldset>
        }).collect_view()}
    </section> }
}

#[component]
pub fn PlanField(
    label: &'static str,
    value: Signal<String>,
    on_value: Callback<String>,
    closed: bool,
    #[prop(optional)] unit: Option<&'static str>,
    #[prop(optional)] field_id: Option<&'static str>,
    #[prop(optional)] formal_term: Option<&'static str>,
    #[prop(optional)] hint: Option<&'static str>,
    #[prop(optional)] example: Option<&'static str>,
    #[prop(optional)] outcome: Option<&'static str>,
    #[prop(default = false)] numeric: bool,
) -> impl IntoView {
    let help_id = field_id.map(|id| format!("{id}-help"));
    let error_id = field_id.map(|id| format!("{id}-error"));
    let has_guidance = hint.is_some() || example.is_some() || outcome.is_some();
    let described_help_id = help_id.clone();
    let described_error_id = error_id.clone();
    let described_by = move || {
        let mut ids = Vec::new();
        if has_guidance && let Some(id) = described_help_id.as_deref() {
            ids.push(id);
        }
        if numeric
            && numeric_input_invalid(&value.get())
            && let Some(id) = described_error_id.as_deref()
        {
            ids.push(id);
        }
        (!ids.is_empty()).then(|| ids.join(" "))
    };
    view! { <label class="guided-field"><span>{label}</span>{formal_term.map(|term| view! { <small class="formal-term">{term}</small> })}{if closed {
        view! { <p class="readonly-value">{move || { let value = value.get(); if value.is_empty() { "—".into() } else if let Some(unit) = unit { format!("{value} {unit}") } else { value } }}</p> }.into_any()
    } else {
        view! { <><span class="input-with-unit"><input id=field_id type="text" inputmode=if numeric { "decimal" } else { "text" } aria-describedby=described_by aria-invalid=move || (numeric && numeric_input_invalid(&value.get())).then_some("true") prop:value=move || value.get() on:input=move |event| on_value.run(event_target_value(&event)) on:blur=move |event| { if numeric { on_value.run(format_numeric_input(&event_target_value(&event))); } }/>{unit.map(|unit| view! { <span class="unit">{unit}</span> })}</span><Show when=move || has_guidance><span id=help_id.clone() class="guided-help">{hint.map(|text| view! { <small class="field-hint">{text}</small> })}{example.map(|text| view! { <small class="field-example">{text}</small> })}{outcome.map(|text| view! { <small class="field-effect">{text}</small> })}</span></Show><Show when=move || numeric && numeric_input_invalid(&value.get())><small id=error_id.clone() class="field-error">"กรุณากรอกเป็นตัวเลข"</small></Show></> }.into_any()
    }}</label> }
}

#[component]
fn LiveTotal(
    form: RwSignal<PlanForm>,
    assets: StoredValue<Vec<calc::AssetAllocation>>,
    starting_capital: Option<rust_decimal::Decimal>,
) -> impl IntoView {
    let summary = move || {
        assets
            .with_value(|assets| live_profit_with_assets(&form.get(), assets, starting_capital))
            .map(|profit| format!("กำไรสุทธิโดยประมาณ {} บาท", money(profit)))
            .unwrap_or_else(|| "ยังคำนวณกำไรสุทธิไม่ได้".into())
    };
    let waiting = move || {
        let count = form.get().unclassified.len();
        (count > 0).then(|| format!("ยังมี {count} รายการที่ยังไม่ได้บอกว่าเป็นแบบไหน จึงยังไม่ถูกนับ"))
    };
    view! { <aside class="live-total" aria-live="polite">
        <strong>{summary}</strong>
        {move || waiting().map(|text| view! { <small>{text}</small> })}
    </aside> }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum NavSection {
    Home,
    Input,
    Analysis,
}

#[component]
pub(crate) fn BottomNav(plan_id: i64, active: NavSection) -> impl IntoView {
    view! { <nav class="bottom-nav" aria-label="เมนูหลัก">
        <A
            attr:class=if active == NavSection::Home { "nav-active" } else { "" }
            attr:aria-current=(active == NavSection::Home).then_some("page")
            href=format!("/plans/{plan_id}/dashboard")
        >"หน้าแรก"</A>
        <A
            attr:class=if active == NavSection::Input { "nav-active" } else { "" }
            attr:aria-current=(active == NavSection::Input).then_some("page")
            href=format!("/plans/{plan_id}")
        >"กรอกข้อมูล"</A>
        <A
            attr:class=if active == NavSection::Analysis { "nav-active" } else { "" }
            attr:aria-current=(active == NavSection::Analysis).then_some("page")
            href=format!("/plans/{plan_id}/analysis")
        >"วิเคราะห์"</A>
        <A href=format!("/history?from={plan_id}")>"ฤดูกาล"</A>
    </nav> }
}

#[component]
fn ValidationSummary(form: RwSignal<PlanForm>, section: String) -> impl IntoView {
    view! { <div class="validation-summary" aria-live="polite">{move || form.get().section_errors(&section).into_iter().next().map(|error| view! { <p><strong>"ตรวจข้อมูลส่วนนี้: "</strong>{error.message}</p> })}</div> }
}

#[component]
fn ServerMessage<S>(action: ServerAction<S>) -> impl IntoView
where
    S: server_fn::ServerFn<Output = String> + Clone + Send + Sync + 'static,
    S::Error: Clone + std::fmt::Display + Send + Sync + 'static,
{
    view! { <p class="form-message" aria-live="polite">{move || action.value().get().map(|result| result.unwrap_or_else(|error| error.to_string()))}</p> }
}

#[component]
fn ServerErrorMessage<S>(action: ServerAction<S>) -> impl IntoView
where
    S: server_fn::ServerFn<Output = ()> + Clone + Send + Sync + 'static,
    S::Error: Clone + std::fmt::Display + Send + Sync + 'static,
{
    view! {
        <p class="form-message bad-message" aria-live="polite">
            {move || action.value().get().and_then(|result| result.err().map(|error| error.to_string()))}
        </p>
    }
}

fn route_plan_id() -> impl Fn() -> Option<i64> + Copy {
    let params = use_params_map();
    move || params.with(|params| params.get("id").and_then(|id| id.parse().ok()))
}

/// The same six decisions, in the same words, on the hub, the dashboard, and
/// the analysis page: what can be answered now, which single fact is still
/// missing, and which optional fact would add a result.
#[component]
pub(crate) fn DecisionList(
    plan_id: i64,
    decisions: Vec<DecisionReadiness>,
    #[prop(optional)] compact: bool,
) -> impl IntoView {
    view! {
        <section class="card decision-list" class:decision-list-compact=compact>
            <h2>"ตอนนี้ตอบได้ว่า"</h2>
            <ul class="decision-rows">
                {decisions.into_iter().map(|readiness| {
                    let question = readiness.decision.question();
                    let formal = readiness.decision.formal_term();
                    let status = match readiness.state {
                        DecisionState::Ready => view! { <span class="status good">"ดูได้แล้ว"</span> }.into_any(),
                        DecisionState::Missing { question, section } => view! {
                            <A attr:class="status warning" href=format!("/plans/{plan_id}/{section}")>{format!("ยังขาด: {question}")}</A>
                        }.into_any(),
                        DecisionState::Optional { unlock, section } => view! {
                            <A attr:class="status muted" href=format!("/plans/{plan_id}/{section}")>{format!("เพิ่มได้: {unlock}")}</A>
                        }.into_any(),
                    };
                    view! {
                        <li class="decision-row">
                            <span class="decision-question"><strong>{question}</strong><small class="formal-term">{formal}</small></span>
                            {status}
                        </li>
                    }
                }).collect_view()}
            </ul>
        </section>
    }
}

pub(crate) fn money(value: rust_decimal::Decimal) -> String {
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

fn season_year_label(year: Option<i32>) -> String {
    year.map_or_else(|| "ยังไม่ระบุปี".into(), |year| format!("ฤดูกาล {year}"))
}

fn season_state(closed: bool, is_latest: bool) -> (&'static str, &'static str) {
    match (closed, is_latest) {
        (true, _) => ("ปิดแล้ว", "status muted"),
        (false, true) => ("กำลังทำ", "status good"),
        (false, false) => ("ปีก่อน · ยังไม่ปิด", "status warning"),
    }
}

/// The live figure on input pages counts the assets the owner has already
/// included in this season, so it matches the dashboard rather than a plan
/// with no assets.
fn live_profit_with_assets(
    form: &PlanForm,
    assets: &[calc::AssetAllocation],
    starting_capital: Option<rust_decimal::Decimal>,
) -> Option<rust_decimal::Decimal> {
    form.to_plan()
        .ok()
        .map(|plan| calc::analyze_with_assets(&plan, assets, starting_capital))
        .and_then(|analysis| analysis.business.net_profit)
}

/// Yearly depreciation from the assets included in this season, `None` when
/// none are included, so pages can say the figure is already counted.
pub(crate) fn included_asset_depreciation(
    assets: &[calc::AssetAllocation],
) -> Option<rust_decimal::Decimal> {
    (!assets.is_empty()).then(|| assets.iter().map(|asset| asset.annual_depreciation).sum())
}

fn numeric_input_invalid(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value
            .replace(',', "")
            .parse::<rust_decimal::Decimal>()
            .is_err()
}

fn format_numeric_input(value: &str) -> String {
    let trimmed = value.trim();
    let normalized = trimmed.replace(',', "");
    let Ok(number) = normalized.parse::<rust_decimal::Decimal>() else {
        return value.to_owned();
    };
    let plain = number.normalize().to_string();
    let (integer, fraction) = plain
        .split_once('.')
        .map_or((plain.as_str(), None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
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
    let integer = grouped.chars().rev().collect::<String>();
    fraction.map_or_else(
        || format!("{sign}{integer}"),
        |fraction| format!("{sign}{integer}.{fraction}"),
    )
}

fn variable_kind_label(kind: VariableCostKind) -> &'static str {
    match kind {
        VariableCostKind::Fertilizer => "ปุ๋ยและธาตุอาหาร",
        VariableCostKind::CropProtection => "สารเคมี/ชีวภัณฑ์",
        VariableCostKind::Water => "น้ำ",
        VariableCostKind::OrchardLabor => "แรงงานดูแลสวน",
        VariableCostKind::Electricity => "ไฟฟ้า",
        VariableCostKind::Fuel => "น้ำมัน",
        VariableCostKind::HarvestLabor => "แรงงานเก็บเกี่ยว",
        VariableCostKind::Transport => "ขนส่ง",
        VariableCostKind::Packing => "คัดแยก/บรรจุภัณฑ์",
        VariableCostKind::Maintenance => "ซ่อมบำรุง",
        VariableCostKind::Other => "อื่นๆ",
    }
}

fn parse_variable_kind(value: &str) -> VariableCostKind {
    match value {
        "fertilizer" => VariableCostKind::Fertilizer,
        "crop-protection" => VariableCostKind::CropProtection,
        "water" => VariableCostKind::Water,
        "orchard-labor" => VariableCostKind::OrchardLabor,
        "electricity" => VariableCostKind::Electricity,
        "fuel" => VariableCostKind::Fuel,
        "harvest-labor" => VariableCostKind::HarvestLabor,
        "transport" => VariableCostKind::Transport,
        "packing" => VariableCostKind::Packing,
        "maintenance" => VariableCostKind::Maintenance,
        _ => VariableCostKind::Other,
    }
}

fn variable_kind_options(selected: VariableCostKind) -> impl IntoView {
    [
        ("fertilizer", VariableCostKind::Fertilizer), ("crop-protection", VariableCostKind::CropProtection),
        ("water", VariableCostKind::Water), ("orchard-labor", VariableCostKind::OrchardLabor),
        ("electricity", VariableCostKind::Electricity), ("fuel", VariableCostKind::Fuel),
        ("harvest-labor", VariableCostKind::HarvestLabor), ("transport", VariableCostKind::Transport),
        ("packing", VariableCostKind::Packing), ("maintenance", VariableCostKind::Maintenance),
        ("other", VariableCostKind::Other),
    ].into_iter().map(|(value, kind)| view! { <option value=value selected=kind == selected>{variable_kind_label(kind)}</option> }).collect_view()
}

fn health_question_label(question: HealthQuestion) -> &'static str {
    match question {
        HealthQuestion::ProfitAndCash => "มีกำไรและเงินสดเพียงพอ",
        HealthQuestion::NextSeasonReserve => "มีเงินสำรองสำหรับฤดูถัดไป",
        HealthQuestion::YieldAndQuality => "ผลผลิตและคุณภาพได้ตามแผน",
        HealthQuestion::LossControl => "ควบคุมผลผลิตสูญเสียได้",
        HealthQuestion::MultipleSalesChannels => "มีช่องทางขายมากกว่าหนึ่งทาง",
        HealthQuestion::PriceVolatility => "รับมือความผันผวนของราคาได้",
        HealthQuestion::ResourceEfficiency => "ใช้ทรัพยากรอย่างคุ้มค่า",
        HealthQuestion::EnvironmentalCare => "ดูแลผลกระทบต่อสิ่งแวดล้อม",
        HealthQuestion::FairAndSafeWork => "แรงงานเป็นธรรมและปลอดภัย",
        HealthQuestion::LaborContinuity => "มีแรงงานต่อเนื่องเพียงพอ",
        HealthQuestion::DownsideSurvival => "สวนอยู่รอดได้เมื่อผลลัพธ์แย่ลง",
        HealthQuestion::ContingencyPlan => "มีแผนสำรองเมื่อเกิดเหตุไม่คาดคิด",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_have_two_decimals_and_thousands_separators() {
        assert_eq!(money(rust_decimal::Decimal::from(834_600)), "834,600.00");
        assert_eq!(money(rust_decimal::Decimal::new(-125, 1)), "-12.50");
    }

    #[test]
    fn numeric_blur_formats_valid_values_and_preserves_invalid_text() {
        assert_eq!(format_numeric_input("12345.60"), "12,345.6");
        assert_eq!(format_numeric_input("สิบ"), "สิบ");
        assert!(numeric_input_invalid("สิบ"));
        assert!(!numeric_input_invalid("12,345.6"));
    }

    #[test]
    fn live_total_uses_the_local_calculation_and_changes_with_input() {
        let sample = PlanForm::from_plan(&calc::workbook_sample());
        let original = live_profit_with_assets(&sample, &[], None).expect("sample has profit");
        let mut changed = sample.clone();
        changed.grades[0].price_per_kg = "120".into();
        assert!(
            live_profit_with_assets(&changed, &[], None).expect("changed sample has profit")
                > original
        );
    }

    #[test]
    fn season_cards_name_open_closed_latest_and_older_states() {
        assert_eq!(season_state(false, true).0, "กำลังทำ");
        assert_eq!(season_state(false, false).0, "ปีก่อน · ยังไม่ปิด");
        assert_eq!(season_state(true, true).0, "ปิดแล้ว");
        assert_eq!(season_state(true, false).0, "ปิดแล้ว");
    }
}
