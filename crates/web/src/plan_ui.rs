use calc::{CashKind, HealthQuestion, VariableCostKind};
use leptos::{form::ActionForm, prelude::*};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    auth::{Logout, current_user_email},
    plan_form::{FixedCostForm, GradeForm, PlanForm, VariableCostForm},
    plans::{
        ClearPlan, ClosePlan, CreateEmptyPlan, CreateSamplePlan, DuplicatePlan, PlanRecord,
        SavePlan, list_plans,
    },
};

const SECTIONS: [(&str, &str, &str); 6] = [
    ("market", "ตลาด", "ลูกค้า ความต้องการ และช่องทางขาย"),
    ("production", "ผลผลิตและเกรด", "พื้นที่ ผลผลิต สูญเสีย และสัดส่วนเกรด"),
    ("variable-costs", "ต้นทุนผันแปร", "รายการที่เปลี่ยนตามการผลิต"),
    ("fixed-costs", "ต้นทุนคงที่", "เงินสด ค่าเสื่อม และฐานลงทุน"),
    ("targets", "เป้าหมาย", "ตัวเลขเปรียบเทียบที่เจ้าของกำหนด"),
    ("health", "สุขภาพสวน", "12 คำถาม 6 มิติ"),
];

#[component]
pub fn PlansPage() -> impl IntoView {
    let logout = ServerAction::<Logout>::new();
    let create_sample = ServerAction::<CreateSamplePlan>::new();
    let create_empty = ServerAction::<CreateEmptyPlan>::new();
    let user = Resource::new(|| (), |_| current_user_email());
    let plans = Resource::new(|| (), |_| list_plans());

    view! {
        <section class="page-stack">
            <header class="page-heading">
                <div>
                    <p class="eyebrow">"พื้นที่ส่วนตัว"</p>
                    <h1>"แผนของฉัน"</h1>
                </div>
                <Suspense fallback=move || view! { <span>"…"</span> }>
                    {move || user.get().map(|result| view! {
                        <span class="user-email">{result.ok().flatten().unwrap_or_default()}</span>
                    })}
                </Suspense>
            </header>

            <section class="card starter-card">
                <h2>"เริ่มจากตัวอย่าง หรือเริ่มว่าง"</h2>
                <p>"แผนตัวอย่างใช้ตัวเลขจากแบบคำนวณเดิม และล้างออกได้ในครั้งเดียว"</p>
                <div class="actions">
                    <ActionForm action=create_sample>
                        <button class="primary" type="submit">"เปิดแผนตัวอย่าง"</button>
                    </ActionForm>
                    <ActionForm action=create_empty>
                        <input type="hidden" name="name" value="แผนฤดูใหม่"/>
                        <button class="secondary" type="submit">"เริ่มแผนว่าง"</button>
                    </ActionForm>
                </div>
            </section>

            <Suspense fallback=move || view! { <p>"กำลังอ่านแผน…"</p> }>
                {move || plans.get().map(|result| match result {
                    Ok(items) if items.is_empty() => view! {
                        <section class="empty-state"><h2>"ยังไม่มีแผน"</h2><p>"เลือกตัวอย่างหรือแผนว่างด้านบนได้เลย"</p></section>
                    }.into_any(),
                    Ok(items) => view! {
                        <section class="plan-list" aria-label="รายการแผน">
                            {items.into_iter().map(|plan| view! {
                                <A attr:class="plan-list-item" href=format!("/plans/{}", plan.id)>
                                    <span><strong>{plan.name}</strong><small>{if plan.closed { "ปิดฤดูกาลแล้ว" } else { "กำลังทำ" }}</small></span>
                                    <span aria-hidden="true">"›"</span>
                                </A>
                            }).collect_view()}
                        </section>
                    }.into_any(),
                    Err(_) => view! { <p class="form-message bad-message">"อ่านรายการแผนไม่ได้ กรุณาลองอีกครั้ง"</p> }.into_any(),
                })}
            </Suspense>

            <ActionForm action=logout>
                <button class="text-button" type="submit">"ออกจากระบบ"</button>
            </ActionForm>
            <BottomNav plan_id=None/>
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
        <Suspense fallback=move || view! { <p>"กำลังอ่านแผน…"</p> }>
            {move || record.get().map(|result| match result {
                Ok(Some(record)) => view! { <PlanHub record/> }.into_any(),
                _ => view! { <section class="card"><h1>"ไม่พบแผนนี้"</h1><A href="/plans">"กลับไปแผนของฉัน"</A></section> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn PlanHub(record: PlanRecord) -> impl IntoView {
    let clear = ServerAction::<ClearPlan>::new();
    let duplicate = ServerAction::<DuplicatePlan>::new();
    let close = ServerAction::<ClosePlan>::new();
    let id = record.id;
    let form = record.form;
    let name = form.name.clone();
    let duplicate_name = next_season_name(&form.name);

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading">
                <div><p class="eyebrow">"ฤดูกาล"</p><h1>{name.clone()}</h1></div>
                <A attr:class="icon-button" href="/plans" attr:aria-label="กลับไปรายการแผน">"×"</A>
            </header>
            <Show when=move || record.closed>
                <div class="closed-banner" role="status">"ฤดูกาลนี้ปิดแล้ว · แก้ไขไม่ได้"</div>
            </Show>
            <section class="section-list">
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
            <LiveTotal form=RwSignal::new(form.clone())/>
            <section class="card plan-actions">
                <h2>"จัดการฤดูกาล"</h2>
                <ActionForm action=duplicate>
                    <input type="hidden" name="id" value=id/>
                    <label><span>"ชื่อสำเนา"</span><input type="text" name="new_name" value=duplicate_name/></label>
                    <button class="secondary" type="submit">{if record.closed { "ทำแผนฤดูถัดไปจากฤดูนี้" } else { "ทำสำเนาฤดูกาล" }}</button>
                </ActionForm>
                <Show when=move || !record.closed>
                    <details class="confirm-box">
                        <summary>"ล้างข้อมูลตัวอย่าง"</summary>
                        <p>"จะลบข้อมูลตลาด ผลผลิต ต้นทุน เป้าหมาย และคำตอบสุขภาพทั้งหมด แต่เก็บชื่อฤดูกาลไว้"</p>
                        <ActionForm action=clear>
                            <input type="hidden" name="id" value=id/>
                            <button class="danger" type="submit">"ยืนยัน ล้างทั้งหมด"</button>
                        </ActionForm>
                    </details>
                    <details class="confirm-box">
                        <summary>"ปิดฤดูกาล"</summary>
                        <p>"เมื่อปิดแล้ว แผนนี้จะอ่านได้อย่างเดียว และแก้กลับไม่ได้"</p>
                        <ActionForm action=close>
                            <input type="hidden" name="id" value=id/>
                            <button class="danger" type="submit">"ยืนยัน ปิดฤดูกาล"</button>
                        </ActionForm>
                    </details>
                </Show>
            </section>
            <BottomNav plan_id=Some(id)/>
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
                    Ok(Some(record)) if SECTIONS.iter().any(|(slug, _, _)| *slug == section) =>
                        view! { <PlanSectionView record section/> }.into_any(),
                    _ => view! { <section class="card"><h1>"ไม่พบส่วนนี้"</h1><A href="/plans">"กลับไปแผนของฉัน"</A></section> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn PlanSectionView(record: PlanRecord, section: String) -> impl IntoView {
    let form = RwSignal::new(record.form);
    let save = ServerAction::<SavePlan>::new();
    let id = record.id;
    let closed = record.closed;
    let title = SECTIONS
        .iter()
        .find(|(slug, _, _)| *slug == section)
        .map(|(_, title, _)| *title)
        .unwrap_or("ข้อมูลแผน");

    view! {
        <section class="page-stack plan-page">
            <header class="page-heading compact-heading">
                <div><p class="eyebrow">"กรอกข้อมูล"</p><h1>{title}</h1></div>
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
            <BottomNav plan_id=Some(id)/>
        </section>
    }
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
        <details class="explanation"><summary>"ⓘ เงินสด กับ ไม่ใช่เงินสด ต่างกันอย่างไร"</summary><h3>"คืออะไร"</h3><p>"ค่าเสื่อมระบบน้ำและค่าเสื่อมรถ เป็นต้นทุนที่ลงบัญชีแต่ปีนี้ไม่ได้ควักเงินจ่าย ส่วนค่าเช่า ดอกเบี้ย และค่าแรงประจำ จ่ายจริงทุกปี"</p><h3>"ใช้ยังไง"</h3><p>"เลือกประเภทให้ถูกตอนกรอกต้นทุนคงที่"</p><h3>"ทำไมต้องมี"</h3><p>"เป็นสิ่งเดียวที่ทำให้กระแสเงินสดกับกำไรสุทธิต่างกันได้"</p><h3>"ไม่ใส่ได้ไหม"</h3><p>"ใส่ผิดประเภทได้ แต่กระแสเงินสดจะผิดตาม"</p></details>
        {move || form.get().fixed_costs.into_iter().enumerate().map(|(index, line)| view! {
            <div class="repeat-row">
                <PlanField label="รายการ" value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.name.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.name = v })) closed/>
                <label><span>"ประเภท"</span>{if closed { view! { <p class="readonly-value">{cash_kind_label(line.cash_kind)}</p> }.into_any() } else { view! { <select on:change=move |event| { let kind = if event_target_value(&event) == "non-cash" { CashKind::NonCash } else { CashKind::Cash }; form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.cash_kind = kind }); }><option value="cash" selected=line.cash_kind == CashKind::Cash>"เงินสด"</option><option value="non-cash" selected=line.cash_kind == CashKind::NonCash>"ไม่ใช่เงินสด"</option></select> }.into_any() }}</label>
                <PlanField label="จำนวนต่อปี" unit="บาท" numeric=true value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.amount_per_year.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.amount_per_year = v })) closed/>
                <PlanField label="ฐานเงินลงทุน" unit="บาท" numeric=true value=Signal::derive(move || form.get().fixed_costs.get(index).map(|l| l.investment_base.clone()).unwrap_or_default()) on_value=Callback::new(move |v| form.update(|f| if let Some(l) = f.fixed_costs.get_mut(index) { l.investment_base = v })) closed/>
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
            .map(|profit| format!("กำไรสุทธิประมาณ {} บาท", money(profit)))
            .unwrap_or_else(|| "กรอกข้อมูลเพิ่มเพื่อคำนวณยอดรวม".into())
    };
    view! { <aside class="live-total" aria-live="polite"><span>"ยอดรวมสด"</span><strong>{summary}</strong><details class="live-explanation"><summary aria-label="อธิบายกำไรสุทธิและกระแสเงินสด">"ⓘ"</summary><div><h3>"คืออะไร"</h3><p>"กำไรสุทธิรวมค่าเสื่อมราคาซึ่งไม่ได้จ่ายเป็นเงินสดจริงในปีนี้ กระแสเงินสดตัดค่าเสื่อมออก จึงเป็นเงินที่เข้ากระเป๋าจริง"</p><h3>"ใช้ยังไง"</h3><p>"ใช้กำไรสุทธิดูว่าธุรกิจกำไรไหม ใช้กระแสเงินสดดูว่าเดือนหน้ามีเงินจ่ายค่าแรงหรือเปล่า"</p><h3>"ทำไมต้องมี"</h3><p>"สวนที่กำไรดีแต่เงินสดขาดมือ ล้มได้ และล้มบ่อย"</p><h3>"ไม่ใส่ได้ไหม"</h3><p>"คำนวณให้เอง แต่ถ้าไม่แยกว่าต้นทุนคงที่ตัวไหนเป็นเงินสด ตัวเลขนี้จะเท่ากับกำไรสุทธิ และจะไม่บอกอะไรเลย"</p></div></details></aside> }
}

#[component]
fn BottomNav(plan_id: Option<i64>) -> impl IntoView {
    let input_href = plan_id
        .map(|id| format!("/plans/{id}"))
        .unwrap_or_else(|| "/plans".into());
    view! { <nav class="bottom-nav" aria-label="เมนูหลัก">
        <A href="/plans">"หน้าแรก"</A>
        <A href=input_href>"กรอกข้อมูล"</A>
        <span class="nav-disabled">"วิเคราะห์"</span>
        <A href="/plans">"บัญชี"</A>
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

fn route_plan_id() -> impl Fn() -> Option<i64> + Copy {
    let params = use_params_map();
    move || params.with(|params| params.get("id").and_then(|id| id.parse().ok()))
}

fn money(value: rust_decimal::Decimal) -> String {
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

fn live_profit(form: &PlanForm) -> Option<rust_decimal::Decimal> {
    form.to_plan()
        .ok()
        .map(|plan| calc::analyze(&plan))
        .and_then(|analysis| analysis.business.net_profit)
}

fn next_season_name(name: &str) -> String {
    let mut parts = name
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if let Some(part) = parts
        .iter_mut()
        .rev()
        .find(|part| part.len() == 4 && part.chars().all(|character| character.is_ascii_digit()))
        && let Ok(year) = part.parse::<u32>()
    {
        *part = (year + 1).to_string();
        return parts.join(" ");
    }
    format!("สำเนา {name}")
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
    fn duplicate_defaults_to_the_next_named_season_when_a_year_is_present() {
        assert_eq!(next_season_name("ฤดู 2569"), "ฤดู 2570");
        assert_eq!(next_season_name("สวนหลัก"), "สำเนา สวนหลัก");
    }
}
