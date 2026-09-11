use calc::{CashKind, HealthQuestion, VariableCostKind};
use leptos::{form::ActionForm, prelude::*};
use leptos_router::{
    components::A,
    hooks::{use_params_map, use_query_map},
};

use crate::{
    analysis_ui::{PlanAnalysisView, PlanDashboardView},
    auth::{Logout, current_user_email},
    plan_form::{FixedCostForm, GradeForm, PlanForm, VariableCostForm},
    plans::{
        ClosePlan, CreateSeason, PlanRecord, SavePlan, SaveQuickStep, SwitchForecastMode,
        UpdateSeasonMetadata, list_plans, quick_resume_path,
    },
};

const SECTIONS: [(&str, &str, &str); 6] = [
    ("market", "ตลาด", "ลูกค้า ความต้องการ และช่องทางขาย"),
    ("production", "ผลผลิตและเกรด", "พื้นที่ ผลผลิต สูญเสีย และสัดส่วนเกรด"),
    ("variable-costs", "ต้นทุนผันแปร", "รายการที่เปลี่ยนตามการผลิต"),
    ("fixed-costs", "ต้นทุนคงที่", "เงินสด ค่าเสื่อม และเงินลงทุน"),
    ("targets", "เป้าหมาย", "ตัวเลขเปรียบเทียบที่เจ้าของกำหนด"),
    ("health", "สุขภาพสวน", "12 คำถาม 6 มิติ"),
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
                        <input type="text" name="season_year" inputmode="numeric" pattern="[0-9]{4}" maxlength="4" value=year placeholder="เช่น 2569" required/>
                    </label>
                    <label>
                        <span>"ชื่อฤดูกาล"</span>
                        <input type="text" name="name" maxlength="120" value=name placeholder="เช่น สวนรวม หมอนทอง" required/>
                    </label>
                    <label>
                        <span>"บันทึก (ไม่บังคับ)"</span>
                        <textarea name="note" maxlength="2000" rows="4" placeholder="เรื่องที่อยากจำเมื่อกลับมาดูฤดูกาลนี้"></textarea>
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
                    label="ผลต่อต้น"
                    unit="ผล"
                    numeric=true
                    value=Signal::derive(move || form.get().production.fruits_per_tree)
                    on_value=Callback::new(move |value| form.update(|form| form.production.fruits_per_tree = value))
                    closed=false
                />
                <PlanField
                    label="ราคาเกรดแรก"
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
                {move || form.get().to_plan().ok().map(|plan| view! {
                    <crate::analysis_ui::DashboardFigures analysis=calc::analyze(&plan)/>
                })}
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
fn PlanHub(record: PlanRecord) -> impl IntoView {
    let close = ServerAction::<ClosePlan>::new();
    let update_metadata = ServerAction::<UpdateSeasonMetadata>::new();
    let id = record.id;
    let form = record.form;
    let name = form.name.clone();
    let year = record.season_year;
    let note = record.note;
    let closed = record.closed;
    let forecast_mode = record.forecast_mode;
    let quick_estimate = record.quick_estimate;

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
                    <DetailedModeHub id form=form.clone() closed/>
                }.into_any(),
            }}
            // No live total here. The bar exists so a figure moves while the
            // owner types, and nothing on this page is typed; it only covered
            // the cards below it. The dashboard carries the figure instead.
            <section class="card plan-actions">
                <h2>"จัดการฤดูกาล"</h2>
                <A attr:class="button secondary" href=format!("/plans/new?source={id}")>"ทำฤดูกาลถัดไปจากฤดูนี้"</A>
                <Show when=move || !closed>
                    <details class="confirm-box">
                        <summary>"ปิดฤดูกาล"</summary>
                        <p>"เมื่อปิดแล้ว ฤดูกาลนี้จะอ่านได้อย่างเดียว และแก้กลับไม่ได้"</p>
                        <ActionForm action=close>
                            <input type="hidden" name="id" value=id/>
                            <button class="danger" type="submit">"ยืนยัน ปิดฤดูกาล"</button>
                        </ActionForm>
                    </details>
                </Show>
            </section>
            <BottomNav plan_id=id active=NavSection::Input/>
        </section>
    }
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
fn DetailedModeHub(id: i64, form: PlanForm, closed: bool) -> impl IntoView {
    let switch = ServerAction::<SwitchForecastMode>::new();
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
        <section class="section-list">
            <A attr:class="section-card" href=format!("/plans/{id}/dashboard")>
                <span><strong>"หน้าแรก"</strong><small>"ตัวเลขสรุปของฤดูกาลนี้"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
            <A attr:class="section-card" href=format!("/plans/{id}/analysis")>
                <span><strong>"วิเคราะห์"</strong><small>"ประสิทธิภาพ ตรวจสอบ ภาษี สถานการณ์"</small></span>
                <span class="status muted">"เปิด"</span>
            </A>
            {SECTIONS.into_iter().map(|(slug, title, description)| {
                let complete = form.section_complete(slug);
                view! {
                    <A attr:class="section-card" href=format!("/plans/{id}/{slug}")>
                        <span><strong>{title}</strong><small>{description}</small></span>
                        <span class=if complete { "status good" } else { "status muted" }>{if complete { "ครบ" } else { "ยังไม่ครบ" }}</span>
                    </A>
                }
            }).collect_view()}
        </section>
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
                        view! { <QuickModeRequired record/> }.into_any(),
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
    let (position, question, hint, unit, value, back) = match step.as_str() {
        "production" => (
            1,
            "ฤดูกาลนี้คาดว่าจะขายทุเรียนได้กี่กิโลกรัม",
            "ใช้ยอดที่คาดว่าจะขายได้จริงหลังหักผลเสียและผลที่ไม่ได้ขาย",
            "กก.",
            record.quick_estimate.sellable_yield_kg,
            format!("/plans/{id}"),
        ),
        "price" => (
            2,
            "คาดว่าจะขายได้ราคาเฉลี่ยกี่บาทต่อกิโลกรัม",
            "ถ้ามีหลายเกรด ให้ใช้ราคาเฉลี่ยคร่าว ๆ ของทั้งฤดูกาล",
            "บาท/กก.",
            record.quick_estimate.average_price_per_kg,
            format!("/plans/{id}/quick/production"),
        ),
        _ => (
            3,
            "คาดว่าฤดูกาลนี้มีต้นทุนรวมประมาณเท่าไร",
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
                        <span>{question}</span>
                        <span class="input-with-unit">
                            <input type="text" name="value" inputmode="decimal" value=value required autofocus/>
                            <span class="unit">{unit}</span>
                        </span>
                        <small class="field-hint">{hint}</small>
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
fn QuickModeRequired(record: PlanRecord) -> impl IntoView {
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
                    Ok(Some(record)) if SECTIONS.iter().any(|(slug, _, _)| *slug == section) =>
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
        return view! { <QuickModeRequired record/> }.into_any();
    }
    let form = RwSignal::new(record.form);
    let save = ServerAction::<SavePlan>::new();
    let id = record.id;
    let closed = record.closed;
    let season_label = format!(
        "{} · {}",
        season_year_label(record.season_year),
        form.get().name
    );
    let title = SECTIONS
        .iter()
        .find(|(slug, _, _)| *slug == section)
        .map(|(_, title, _)| *title)
        .unwrap_or("ข้อมูลแผน");

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
                <input type="hidden" name="form_json" value=move || serde_json::to_string(&form.get()).unwrap_or_default()/>
                <SectionFields section=section.clone() form closed/>
                <ValidationSummary form/>
                <Show when=move || !closed>
                    <button class="primary save-button" type="submit" disabled=move || form.get().to_plan().is_err()>"บันทึกส่วนนี้"</button>
                </Show>
            </ActionForm>
            <ServerMessage action=save/>
            <LiveTotal form/>
            <BottomNav plan_id=id active=NavSection::Input/>
        </section>
    }.into_any()
}

#[component]
fn SectionFields(section: String, form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    match section.as_str() {
        "market" => view! { <MarketFields form closed/> }.into_any(),
        "production" => view! { <ProductionFields form closed/> }.into_any(),
        "variable-costs" => view! { <VariableCostFields form closed/> }.into_any(),
        "fixed-costs" => view! { <FixedCostFields form closed/> }.into_any(),
        "targets" => view! { <TargetFields form closed/> }.into_any(),
        "health" => view! { <HealthFields form closed/> }.into_any(),
        _ => view! { <p>"ไม่พบข้อมูลส่วนนี้"</p> }.into_any(),
    }
}

#[component]
fn MarketFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <PlanField label="ลูกค้าเป้าหมาย" value=Signal::derive(move || form.get().market.target_customer) on_value=Callback::new(move |v| form.update(|f| f.market.target_customer = v)) closed/>
        <PlanField label="ความต้องการของตลาด" unit="กก." numeric=true value=Signal::derive(move || form.get().market.demand_kg) on_value=Callback::new(move |v| form.update(|f| f.market.demand_kg = v)) closed/>
        <PlanField label="ราคาขั้นต่ำที่รับได้" unit="บาท/กก." numeric=true value=Signal::derive(move || form.get().market.minimum_price_per_kg) on_value=Callback::new(move |v| form.update(|f| f.market.minimum_price_per_kg = v)) closed/>
        <PlanField label="ช่วงเวลาขาย" value=Signal::derive(move || form.get().market.sales_period) on_value=Callback::new(move |v| form.update(|f| f.market.sales_period = v)) closed/>
        <PlanField label="จำนวนช่องทางขาย" unit="ช่องทาง" numeric=true value=Signal::derive(move || form.get().market.sales_channels) on_value=Callback::new(move |v| form.update(|f| f.market.sales_channels = v)) closed/>
        <PlanField label="สัดส่วนลูกค้ารายใหญ่ที่สุด" unit="%" numeric=true value=Signal::derive(move || form.get().market.largest_buyer_percent) on_value=Callback::new(move |v| form.update(|f| f.market.largest_buyer_percent = v)) closed/>
        <PlanField label="ข้อกำหนดคุณภาพ" value=Signal::derive(move || form.get().market.quality_requirements) on_value=Callback::new(move |v| form.update(|f| f.market.quality_requirements = v)) closed/>
    </section> }
}

#[component]
fn ProductionFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <div class="field-stack">
        <section class="card field-stack">
            <PlanField label="พื้นที่ให้ผลผลิต" unit="ไร่" numeric=true value=Signal::derive(move || form.get().production.area_rai) on_value=Callback::new(move |v| form.update(|f| f.production.area_rai = v)) closed/>
            <PlanField label="ต้นที่ให้ผลผลิต" unit="ต้น" numeric=true value=Signal::derive(move || form.get().production.producing_trees) on_value=Callback::new(move |v| form.update(|f| f.production.producing_trees = v)) closed/>
            <PlanField label="ผลต่อต้น" unit="ผล" numeric=true value=Signal::derive(move || form.get().production.fruits_per_tree) on_value=Callback::new(move |v| form.update(|f| f.production.fruits_per_tree = v)) closed/>
            <PlanField label="น้ำหนักเฉลี่ยต่อผล" unit="กก." numeric=true value=Signal::derive(move || form.get().production.average_fruit_weight_kg) on_value=Callback::new(move |v| form.update(|f| f.production.average_fruit_weight_kg = v)) closed/>
            <PlanField label="สัดส่วนสูญเสีย" unit="%" numeric=true value=Signal::derive(move || form.get().production.loss_percent) on_value=Callback::new(move |v| form.update(|f| f.production.loss_percent = v)) closed/>
        </section>
        <section class="card field-stack">
            <div class="section-title"><div><h2>"สัดส่วนเกรด"</h2><p>"เพิ่มหรือลดได้สูงสุด 10 เกรด"</p></div><strong class=move || if form.get().grade_total_percent() == Some(rust_decimal::Decimal::ONE_HUNDRED) { "good-text" } else { "bad-text" }>{move || form.get().grade_total_percent().map(|v| format!("{}%", v.normalize())).unwrap_or_else(|| "กรอกสัดส่วน".into())}</strong></div>
            {move || form.get().grades.into_iter().enumerate().map(|(index, grade)| view! {
                <div class="repeat-row">
                    <PlanField label="ชื่อเกรด" value=Signal::derive(move || form.get().grades.get(index).map(|g| g.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.name = v })) closed/>
                    <PlanField label="สัดส่วน" unit="%" numeric=true value=Signal::derive(move || form.get().grades.get(index).map(|g| g.share_percent.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.share_percent = v })) closed/>
                    <PlanField label="ราคาขาย" unit="บาท/กก." numeric=true value=Signal::derive(move || form.get().grades.get(index).map(|g| g.price_per_kg.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.price_per_kg = v })) closed/>
                    <label class="check-field"><input type="checkbox" prop:checked=grade.counts_as_quality_grade disabled=closed on:change=move |event| form.update(|f| if let Some(g) = f.grades.get_mut(index) { g.counts_as_quality_grade = event_target_checked(&event) })/><span>"นับเป็นเกรดคุณภาพ"</span></label>
                    <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.grades.len() { f.grades.remove(index); } })>"ลบเกรดนี้"</button></Show>
                </div>
            }).collect_view()}
            <Show when=move || !closed && form.get().grades.len() < 10><button class="secondary" type="button" on:click=move |_| form.update(|f| f.grades.push(GradeForm::default()))>"+ เพิ่มเกรด"</button></Show>
        </section>
    </div> }
}

#[component]
fn VariableCostFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <p class="section-intro">"รายการเก็บเกี่ยว ขนส่ง และบรรจุภัณฑ์จะใช้ปริมาณผลผลิตขายได้ ถ้าเว้นจำนวนไว้"</p>
        {move || form.get().variable_costs.into_iter().enumerate().map(|(index, line)| view! {
            <div class="repeat-row">
                <PlanField label="รายการ" value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.name = v })) closed/>
                <label><span>"ประเภท"</span>{if closed { view! { <p class="readonly-value">{variable_kind_label(line.kind)}</p> }.into_any() } else { view! { <select on:change=move |event| { let kind = parse_variable_kind(&event_target_value(&event)); form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.kind = kind }); }>{variable_kind_options(line.kind)}</select> }.into_any() }}</label>
                <PlanField label="จำนวน" numeric=true value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.quantity.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.quantity = v })) closed/>
                <PlanField label="หน่วย" value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.unit.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.unit = v })) closed/>
                <PlanField label="ราคาต่อหน่วย" unit="บาท" numeric=true value=Signal::derive(move || form.get().variable_costs.get(index).map(|l| l.unit_price.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.variable_costs.get_mut(index) { l.unit_price = v })) closed/>
                <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.variable_costs.len() { f.variable_costs.remove(index); } })>"ลบรายการ"</button></Show>
            </div>
        }).collect_view()}
        <Show when=move || !closed><button class="secondary" type="button" on:click=move |_| form.update(|f| f.variable_costs.push(VariableCostForm::default()))>"+ เพิ่มต้นทุน"</button></Show>
    </section> }
}

#[component]
fn FixedCostFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <p class="section-intro">"กรอกเงินลงทุนเฉพาะรายการที่ใช้เงินก้อนซื้อหรือสร้างสิ่งที่ใช้ได้หลายปี เช่น ระบบน้ำ รถ หรือเครื่องมือ ค่าใช้จ่ายประจำที่ไม่มีเงินก้อนเริ่มต้น เว้นช่องนี้ได้"</p>
        <details class="explanation"><summary>"ⓘ เงินสด กับ ไม่ใช่เงินสด ต่างกันอย่างไร"</summary><h3>"คืออะไร"</h3><p>"ค่าเสื่อมระบบน้ำและค่าเสื่อมรถ เป็นต้นทุนที่ลงบัญชีแต่ปีนี้ไม่ได้ควักเงินจ่าย ส่วนค่าเช่า ดอกเบี้ย และค่าแรงประจำ จ่ายจริงทุกปี"</p><h3>"ใช้ยังไง"</h3><p>"เลือกประเภทให้ถูกตอนกรอกต้นทุนคงที่"</p><h3>"ทำไมต้องมี"</h3><p>"เป็นสิ่งเดียวที่ทำให้กระแสเงินสดกับกำไรสุทธิต่างกันได้"</p><h3>"ไม่ใส่ได้ไหม"</h3><p>"ใส่ผิดประเภทได้ แต่กระแสเงินสดจะผิดตาม"</p></details>
        {move || form.get().fixed_costs.into_iter().enumerate().map(|(index, line)| view! {
            <div class="repeat-row">
                <PlanField label="รายการ" value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.name = v })) closed/>
                <label><span>"ประเภท"</span>{if closed { view! { <p class="readonly-value">{cash_kind_label(line.cash_kind)}</p> }.into_any() } else { view! { <select on:change=move |event| { let kind = if event_target_value(&event) == "non-cash" { CashKind::NonCash } else { CashKind::Cash }; form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.cash_kind = kind }); }><option value="cash" selected=line.cash_kind == CashKind::Cash>"เงินสด"</option><option value="non-cash" selected=line.cash_kind == CashKind::NonCash>"ไม่ใช่เงินสด"</option></select> }.into_any() }}</label>
                <PlanField label="จำนวนต่อปี" unit="บาท" numeric=true value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.amount_per_year.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.amount_per_year = v })) closed/>
                <PlanField label="เงินที่ลงทุนกับรายการนี้ (ถ้ามี)" unit="บาท" numeric=true value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.investment_base.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.investment_base = v })) closed/>
                <Show when=move || !closed><button class="text-button bad-text" type="button" on:click=move |_| form.update(|f| { if index < f.fixed_costs.len() { f.fixed_costs.remove(index); } })>"ลบรายการ"</button></Show>
            </div>
        }).collect_view()}
        <Show when=move || !closed><button class="secondary" type="button" on:click=move |_| form.update(|f| f.fixed_costs.push(FixedCostForm::default()))>"+ เพิ่มต้นทุนคงที่"</button></Show>
    </section> }
}

#[component]
fn TargetFields(form: RwSignal<PlanForm>, closed: bool) -> impl IntoView {
    view! { <section class="card field-stack">
        <p class="section-intro">"เป้าหมายเป็นตัวเลขของเจ้าของสวน ระบบจะไม่เดาให้ ถ้าเว้นว่าง หน้าวิเคราะห์จะแสดงว่ายังไม่ได้ตั้งเป้า"</p>
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
fn PlanField(
    label: &'static str,
    value: Signal<String>,
    on_value: Callback<String>,
    closed: bool,
    #[prop(optional)] unit: Option<&'static str>,
    #[prop(default = false)] numeric: bool,
) -> impl IntoView {
    view! { <label><span>{label}</span>{if closed {
        view! { <p class="readonly-value">{move || { let value = value.get(); if value.is_empty() { "—".into() } else if let Some(unit) = unit { format!("{value} {unit}") } else { value } }}</p> }.into_any()
    } else {
        view! { <><span class="input-with-unit"><input type="text" inputmode=if numeric { "decimal" } else { "text" } prop:value=move || value.get() on:input=move |event| on_value.run(event_target_value(&event)) on:blur=move |event| { if numeric { on_value.run(format_numeric_input(&event_target_value(&event))); } }/>{unit.map(|unit| view! { <span class="unit">{unit}</span> })}</span><Show when=move || numeric && numeric_input_invalid(&value.get())><small class="field-error">"กรุณากรอกเป็นตัวเลข"</small></Show></> }.into_any()
    }}</label> }
}

#[component]
fn LiveTotal(form: RwSignal<PlanForm>) -> impl IntoView {
    let summary = move || {
        live_profit(&form.get())
            .map(|profit| format!("กำไรสุทธิโดยประมาณ {} บาท", money(profit)))
            .unwrap_or_else(|| "ยังคำนวณกำไรสุทธิไม่ได้".into())
    };
    view! { <aside class="live-total" aria-live="polite">
        <strong>{summary}</strong>
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
        <A href=format!("/plans?from={plan_id}")>"ฤดูกาล"</A>
    </nav> }
}

#[component]
fn ValidationSummary(form: RwSignal<PlanForm>) -> impl IntoView {
    view! { <div class="validation-summary" aria-live="polite">{move || form.get().to_plan().err().and_then(|errors| errors.first().cloned()).map(|error| view! { <p><strong>"ตรวจข้อมูล: "</strong>{error.message}</p> })}</div> }
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

fn live_profit(form: &PlanForm) -> Option<rust_decimal::Decimal> {
    form.to_plan()
        .ok()
        .map(|plan| calc::analyze(&plan))
        .and_then(|analysis| analysis.business.net_profit)
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

fn cash_kind_label(kind: CashKind) -> &'static str {
    match kind {
        CashKind::Cash => "เงินสด",
        CashKind::NonCash => "ไม่ใช่เงินสด",
    }
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
        let original = live_profit(&sample).expect("sample has profit");
        let mut changed = sample.clone();
        changed.grades[0].price_per_kg = "120".into();
        assert!(live_profit(&changed).expect("changed sample has profit") > original);
    }

    #[test]
    fn season_cards_name_open_closed_latest_and_older_states() {
        assert_eq!(season_state(false, true).0, "กำลังทำ");
        assert_eq!(season_state(false, false).0, "ปีก่อน · ยังไม่ปิด");
        assert_eq!(season_state(true, true).0, "ปิดแล้ว");
        assert_eq!(season_state(true, false).0, "ปิดแล้ว");
    }
}
