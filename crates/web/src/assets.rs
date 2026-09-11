use leptos::{form::ActionForm, prelude::*};
use leptos_router::{components::A, hooks::use_params_map};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetRow {
    pub id: i64,
    pub facts: calc::AssetFacts,
    pub selected: bool,
    pub allocation: Option<calc::AssetAllocation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetPageData {
    pub plan_id: i64,
    pub plan_name: String,
    pub season_year: Option<i32>,
    pub closed: bool,
    pub forecast_mode: calc::ForecastMode,
    pub starting_capital: Option<rust_decimal::Decimal>,
    pub manual_fixed_cost: Option<rust_decimal::Decimal>,
    pub manual_investment_base: Option<rust_decimal::Decimal>,
    pub assets: Vec<AssetRow>,
}

#[server]
pub async fn load_asset_page(plan_id: i64) -> Result<Option<AssetPageData>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let Some(stored) = store::plans::load(&pool, owner_id, plan_id)
        .await
        .map_err(public_store_error)?
    else {
        return Ok(None);
    };
    let base = calc::analyze(&stored.plan);
    let assets = if stored.closed {
        stored
            .asset_allocations
            .iter()
            .enumerate()
            .map(|(index, allocation)| AssetRow {
                id: -(i64::try_from(index).unwrap_or_default() + 1),
                facts: allocation.facts.clone(),
                selected: true,
                allocation: Some(allocation.clone()),
            })
            .collect()
    } else {
        store::assets::choices_for_plan(&pool, owner_id, plan_id)
            .await
            .map_err(public_store_error)?
            .into_iter()
            .map(|choice| AssetRow {
                id: choice.asset.id,
                facts: choice.asset.facts,
                selected: choice.selected,
                allocation: choice.allocation,
            })
            .collect()
    };
    Ok(Some(AssetPageData {
        plan_id,
        plan_name: stored.plan.name,
        season_year: stored.season_year,
        closed: stored.closed,
        forecast_mode: stored.forecast_mode,
        starting_capital: stored.starting_capital,
        manual_fixed_cost: base.cost.manual_fixed_cost,
        manual_investment_base: base.cost.manual_investment_base,
        assets,
    }))
}

// The server-form boundary intentionally keeps one named argument per native
// HTML field so the journey works without client-side JavaScript.
#[allow(clippy::too_many_arguments)]
#[server]
pub async fn create_owner_asset(
    plan_id: i64,
    name: String,
    kind: String,
    original_cost: String,
    start_year: String,
    approximate_years_in_use: String,
    useful_life_years: String,
    residual_value: String,
    retired_year: String,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let plan = store::plans::load(&pool, owner_id, plan_id)
        .await
        .map_err(public_store_error)?
        .ok_or_else(|| ServerFnError::new("ไม่พบฤดูกาลนี้"))?;
    if plan.closed {
        return Err(ServerFnError::new("ฤดูกาลนี้ปิดแล้วและแก้ไขไม่ได้"));
    }
    let facts = parse_asset(
        &name,
        &kind,
        &original_cost,
        &start_year,
        &approximate_years_in_use,
        &useful_life_years,
        &residual_value,
        &retired_year,
        plan.season_year,
    )?;
    store::assets::create(&pool, owner_id, &facts)
        .await
        .map_err(public_store_error)?;
    leptos_axum::redirect(&format!("/plans/{plan_id}/assets"));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[server]
pub async fn update_owner_asset(
    plan_id: i64,
    asset_id: i64,
    name: String,
    kind: String,
    original_cost: String,
    start_year: String,
    useful_life_years: String,
    residual_value: String,
    retired_year: String,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    ensure_open(&pool, owner_id, plan_id).await?;
    let facts = parse_asset(
        &name,
        &kind,
        &original_cost,
        &start_year,
        "",
        &useful_life_years,
        &residual_value,
        &retired_year,
        None,
    )?;
    store::assets::update(&pool, owner_id, asset_id, &facts)
        .await
        .map_err(public_store_error)?;
    leptos_axum::redirect(&format!("/plans/{plan_id}/assets"));
    Ok(())
}

#[server]
pub async fn set_season_asset(
    plan_id: i64,
    asset_id: i64,
    selection: String,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let selected = match selection.as_str() {
        "include" => true,
        "exclude" => false,
        _ => return Err(ServerFnError::new("การเลือกสินทรัพย์ไม่ถูกต้อง")),
    };
    store::assets::set_selected(&pool, owner_id, plan_id, asset_id, selected)
        .await
        .map_err(public_store_error)?;
    leptos_axum::redirect(&format!("/plans/{plan_id}/assets"));
    Ok(())
}

#[server]
pub async fn save_starting_capital(
    plan_id: i64,
    starting_capital: String,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let value = optional_money(&starting_capital, "เงินทุนเริ่มต้น", false)?;
    store::plans::save_starting_capital(&pool, owner_id, plan_id, value)
        .await
        .map_err(public_store_error)?;
    leptos_axum::redirect(&format!("/plans/{plan_id}/assets"));
    Ok(())
}

#[component]
pub fn AssetPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|params| params.get("id").and_then(|id| id.parse::<i64>().ok()));
    let data = Resource::new(id, |id| async move {
        match id {
            Some(id) => load_asset_page(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Suspense fallback=move || view! { <p>"กำลังอ่านสินทรัพย์…"</p> }>
            {move || data.get().map(|result| match result {
                Ok(Some(data)) => view! { <AssetPageView data/> }.into_any(),
                _ => view! { <section class="card"><h1>"ไม่พบฤดูกาลนี้"</h1><A href="/plans">"กลับไปฤดูกาลของฉัน"</A></section> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
pub fn AssetPageView(data: AssetPageData) -> impl IntoView {
    let create = ServerAction::<CreateOwnerAsset>::new();
    let capital = ServerAction::<SaveStartingCapital>::new();
    let id = data.plan_id;
    let closed = data.closed;
    let year_label = data
        .season_year
        .map_or_else(|| "ฤดูกาล".into(), |year| format!("ฤดูกาล {year}"));
    let selected_depreciation: rust_decimal::Decimal = data
        .assets
        .iter()
        .filter(|asset| asset.selected)
        .filter_map(|asset| asset.allocation.as_ref())
        .map(|allocation| allocation.annual_depreciation)
        .sum();
    let selected_investment: rust_decimal::Decimal = data
        .assets
        .iter()
        .filter(|asset| asset.selected)
        .filter_map(|asset| asset.allocation.as_ref())
        .map(|allocation| allocation.investment_value)
        .sum();

    view! {
        <section class="page-stack asset-page">
            <header class="page-heading">
                <div><p class="eyebrow">{year_label}</p><h1>"สินทรัพย์และเงินลงทุน"</h1><p>{data.plan_name}</p></div>
                <A attr:class="icon-button" href=format!("/plans/{id}") attr:aria-label="กลับหน้าฤดูกาล">"×"</A>
            </header>
            <Show when=move || closed>
                <div class="closed-banner" role="status">"ข้อมูลนี้ถูกเก็บพร้อมตอนปิดฤดู · แก้ไขไม่ได้"</div>
            </Show>
            <Show when=move || data.forecast_mode == calc::ForecastMode::Quick>
                <section class="card asset-mode-note">
                    <strong>"ไม่รวมในประมาณการเร็ว"</strong>
                    <p>"สินทรัพย์และเงินทุนด้านล่างมีผลเมื่อใช้แผนละเอียดเท่านั้น ประมาณการเร็วยังคงใช้ 3 คำตอบเดิม"</p>
                </section>
            </Show>
            <section class="card role-summary">
                <h2>"แยกที่มาของตัวเลข"</h2>
                <dl class="asset-totals">
                    <div><dt>"ต้นทุนคงที่ที่กรอกเอง"</dt><dd>{money_or_dash(data.manual_fixed_cost)}</dd></div>
                    <div><dt>"ค่าเสื่อมจากสินทรัพย์ที่เลือก"</dt><dd>{format!("{} บาท/ปี", money(selected_depreciation))}</dd></div>
                    <div><dt>"เงินลงทุนที่กรอกในรายการเดิม"</dt><dd>{money_or_dash(data.manual_investment_base)}</dd></div>
                    <div><dt>"มูลค่าสินทรัพย์ที่เลือก"</dt><dd>{format!("{} บาท", money(selected_investment))}</dd></div>
                    <div><dt>"เงินทุนเริ่มต้น"</dt><dd>{money_or_dash(data.starting_capital)}</dd></div>
                </dl>
                <p class="warning-copy">"ก่อนเลือกสินทรัพย์ ตรวจว่าคุณไม่ได้กรอกค่าเสื่อมหรือเงินลงทุนของชิ้นเดียวกันไว้ในต้นทุนคงที่แล้ว ระบบจะไม่เดาหรือลบรายการเดิมให้"</p>
                <A attr:class="text-button" href=format!("/plans/{id}/fixed-costs")>"ตรวจต้นทุนคงที่ที่กรอกเอง"</A>
            </section>

            <section class="card">
                <h2>"สินทรัพย์ของฉัน"</h2>
                <p>"บันทึกครั้งเดียว แล้วเลือกใช้แยกในแต่ละฤดู ค่าเสื่อมเป็นเพียงค่าประมาณเพื่อวางแผน ไม่ใช่ค่าเสื่อมทางภาษีหรือราคาตลาด"</p>
                {if data.assets.is_empty() {
                    view! { <p class="muted">"ยังไม่มีสินทรัพย์"</p> }.into_any()
                } else {
                    view! { <div class="asset-list">{data.assets.into_iter().map(|asset| view! { <AssetCard plan_id=id asset closed/> }).collect_view()}</div> }.into_any()
                }}
            </section>

            <Show when=move || !closed>
                <details class="card asset-create">
                    <summary>"+ เพิ่มสินทรัพย์"</summary>
                    <ActionForm action=create>
                        <input type="hidden" name="plan_id" value=id/>
                        <AssetFields facts=None create=true/>
                        <button class="primary" type="submit">"บันทึกสินทรัพย์"</button>
                    </ActionForm>
                    <ActionError action=create/>
                </details>
                <section class="card">
                    <h2>"เงินทุนเริ่มต้น (ไม่บังคับ)"</h2>
                    <p>"ใช้เป็นฐานคำนวณ ROI และระยะคืนทุนเท่านั้น ไม่ใช่ต้นทุนของฤดู และไม่ใช่มูลค่าสินทรัพย์"</p>
                    <ActionForm action=capital>
                        <input type="hidden" name="plan_id" value=id/>
                        <label><span>"เงินทุนเริ่มต้น"</span><span class="input-with-unit"><input type="text" inputmode="decimal" name="starting_capital" value=decimal_input(data.starting_capital)/><span class="unit">"บาท"</span></span><small class="field-hint">"เว้นว่างเพื่อลบค่าเดิม"</small></label>
                        <button class="secondary" type="submit">"บันทึกเงินทุนเริ่มต้น"</button>
                    </ActionForm>
                    <ActionError action=capital/>
                </section>
            </Show>
            <A attr:class="button secondary" href=format!("/plans/{id}")>"กลับหน้าฤดูกาล"</A>
        </section>
    }
}

#[component]
fn AssetCard(plan_id: i64, asset: AssetRow, closed: bool) -> impl IntoView {
    let select = ServerAction::<SetSeasonAsset>::new();
    let update = ServerAction::<UpdateOwnerAsset>::new();
    let facts = asset.facts.clone();
    let allocation = asset.allocation.clone();
    let active = allocation.is_some();
    let selected = asset.selected;
    let asset_id = asset.id;
    let status = match (asset.selected, active) {
        (true, true) => "รวมในฤดูนี้",
        (true, false) => "เลือกไว้ แต่ไม่อยู่ในปีใช้งาน",
        (false, true) => "ยังไม่รวม",
        (false, false) => "ไม่อยู่ในปีใช้งาน",
    };
    view! {
        <article class="repeat-row asset-row">
            <div class="section-title">
                <div><h2>{facts.name.clone()}</h2><p>{asset_kind_label(facts.kind)}</p></div>
                <span class=if asset.selected && active { "status good" } else if asset.selected { "status warning" } else { "status muted" }>{status}</span>
            </div>
            <dl class="asset-facts">
                <div><dt>"ราคาซื้อ/มูลค่าเดิม"</dt><dd>{format!("{} บาท", money(facts.original_cost))}</dd></div>
                <div><dt>"เริ่มใช้ปี พ.ศ."</dt><dd>{facts.start_year}</dd></div>
                {allocation.as_ref().map(|allocation| view! {
                    <div><dt>"ค่าเสื่อมที่เพิ่มในฤดูนี้"</dt><dd>{format!("{} บาท/ปี", money(allocation.annual_depreciation))}</dd></div>
                })}
            </dl>
            {allocation.as_ref().filter(|allocation| allocation.residual_assumed_zero).map(|_| view! {
                <p class="caption">"ไม่ได้ระบุมูลค่าคงเหลือ ระบบใช้ 0 บาทเฉพาะค่าประมาณนี้"</p>
            })}
            {if closed {
                ().into_any()
            } else {
                view! {
                    {if active || selected {
                        view! {
                            <ActionForm action=select>
                                <input type="hidden" name="plan_id" value=plan_id/>
                                <input type="hidden" name="asset_id" value=asset_id/>
                                <input type="hidden" name="selection" value=if selected { "exclude" } else { "include" }/>
                                <button class=if selected { "secondary" } else { "primary" } type="submit">
                                    {if selected { "เอาออกจากฤดูนี้" } else { "รวมในฤดูนี้" }}
                                </button>
                            </ActionForm>
                            <ActionError action=select/>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                    <details class="asset-edit">
                        <summary>"แก้ข้อมูลหรือระบุปีเลิกใช้"</summary>
                        <ActionForm action=update>
                            <input type="hidden" name="plan_id" value=plan_id/>
                            <input type="hidden" name="asset_id" value=asset.id/>
                            <AssetFields facts=Some(facts.clone()) create=false/>
                            <button class="secondary" type="submit">"บันทึกการแก้ไข"</button>
                        </ActionForm>
                        <ActionError action=update/>
                    </details>
                }.into_any()
            }}
        </article>
    }
}

#[component]
fn AssetFields(facts: Option<calc::AssetFacts>, create: bool) -> impl IntoView {
    let facts = facts.unwrap_or(calc::AssetFacts {
        name: String::new(),
        kind: calc::AssetKind::Equipment,
        original_cost: rust_decimal::Decimal::ZERO,
        start_year: 0,
        useful_life_years: None,
        residual_value: None,
        retired_year: None,
    });
    view! {
        <label><span>"ชื่อสินทรัพย์"</span><input type="text" name="name" maxlength="120" value=facts.name required/></label>
        <label><span>"ประเภท"</span><select name="kind"><option value="equipment" selected=facts.kind == calc::AssetKind::Equipment>"อุปกรณ์/สิ่งปลูกสร้าง"</option><option value="owned_land" selected=facts.kind == calc::AssetKind::OwnedLand>"ที่ดินที่เป็นเจ้าของ"</option></select><small class="field-hint">"ที่ดินที่เป็นเจ้าของนับเป็นเงินลงทุน แต่ไม่มีค่าเสื่อม ที่ดินเช่าให้กรอกเป็นต้นทุนคงที่"</small></label>
        <label><span>"ราคาซื้อหรือมูลค่าเดิม"</span><span class="input-with-unit"><input type="text" inputmode="decimal" name="original_cost" value=if facts.original_cost.is_zero() { String::new() } else { facts.original_cost.normalize().to_string() } required/><span class="unit">"บาท"</span></span></label>
        <label><span>"เริ่มใช้ปี พ.ศ."</span><input type="text" inputmode="numeric" pattern="[0-9]{4}" maxlength="4" name="start_year" value=if facts.start_year == 0 { String::new() } else { facts.start_year.to_string() }/></label>
        <Show when=move || create>
            <label><span>"หรือ ใช้มาแล้วประมาณกี่ปี ณ ฤดูนี้"</span><input type="text" inputmode="numeric" name="approximate_years_in_use"/><small class="field-hint">"กรอกอย่างใดอย่างหนึ่ง ระบบคำนวณจากปีฤดู ไม่ใช้วันที่ปัจจุบัน"</small></label>
        </Show>
        <label><span>"อายุใช้งานที่คาด"</span><span class="input-with-unit"><input type="text" inputmode="numeric" name="useful_life_years" value=facts.useful_life_years.map(|value| value.to_string()).unwrap_or_default()/><span class="unit">"ปี"</span></span><small class="field-hint">"ที่ดินที่เป็นเจ้าของให้เว้นว่าง"</small></label>
        <label><span>"มูลค่าคงเหลือที่คาด (ไม่บังคับ)"</span><span class="input-with-unit"><input type="text" inputmode="decimal" name="residual_value" value=decimal_input(facts.residual_value)/><span class="unit">"บาท"</span></span><small class="field-hint">"ถ้าเว้นว่าง ระบบใช้ 0 บาทเฉพาะการวางแผน"</small></label>
        <label><span>"เลิกใช้ตั้งแต่ปี พ.ศ. (ไม่บังคับ)"</span><input type="text" inputmode="numeric" pattern="[0-9]{4}" maxlength="4" name="retired_year" value=facts.retired_year.map(|year| year.to_string()).unwrap_or_default()/></label>
    }
}

#[component]
fn ActionError<S>(action: ServerAction<S>) -> impl IntoView
where
    S: server_fn::ServerFn<Output = ()> + Clone + Send + Sync + 'static,
    S::Error: Clone + std::fmt::Display + Send + Sync + 'static,
{
    view! { <p class="form-message bad-message" aria-live="polite">{move || action.value().get().and_then(|result| result.err().map(|error| error.to_string()))}</p> }
}

#[cfg(feature = "ssr")]
async fn authenticated_owner() -> Result<(sqlx::PgPool, store::users::UserId), ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    let Some(user) = auth_session.user else {
        return Err(ServerFnError::new("กรุณาเข้าสู่ระบบ"));
    };
    Ok((auth_session.backend.pool().clone(), user.id))
}

#[cfg(feature = "ssr")]
async fn ensure_open(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    plan_id: i64,
) -> Result<(), ServerFnError> {
    match store::plans::load(pool, owner_id, plan_id)
        .await
        .map_err(public_store_error)?
    {
        None => Err(ServerFnError::new("ไม่พบฤดูกาลนี้")),
        Some(plan) if plan.closed => Err(ServerFnError::new("ฤดูกาลนี้ปิดแล้วและแก้ไขไม่ได้")),
        Some(_) => Ok(()),
    }
}

#[cfg(feature = "ssr")]
fn public_store_error(error: store::StoreError) -> ServerFnError {
    match error {
        store::StoreError::NotFound => ServerFnError::new("ไม่พบข้อมูลนี้"),
        store::StoreError::Closed => ServerFnError::new("ฤดูกาลนี้ปิดแล้วและแก้ไขไม่ได้"),
        store::StoreError::InvalidValue { .. } => {
            ServerFnError::new("ข้อมูลสินทรัพย์ไม่ถูกต้อง กรุณาตรวจตัวเลขและปี")
        }
        _ => ServerFnError::new("ระบบยังทำรายการไม่ได้ กรุณาลองอีกครั้ง"),
    }
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
fn parse_asset(
    name: &str,
    kind: &str,
    original_cost: &str,
    start_year: &str,
    approximate_years_in_use: &str,
    useful_life_years: &str,
    residual_value: &str,
    retired_year: &str,
    season_year: Option<i32>,
) -> Result<calc::AssetFacts, ServerFnError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(ServerFnError::new("กรุณากรอกชื่อสินทรัพย์ไม่เกิน 120 ตัวอักษร"));
    }
    let kind = match kind {
        "equipment" => calc::AssetKind::Equipment,
        "owned_land" => calc::AssetKind::OwnedLand,
        _ => return Err(ServerFnError::new("ประเภทสินทรัพย์ไม่ถูกต้อง")),
    };
    let explicit_start = optional_year(start_year, "ปีเริ่มใช้")?;
    let prior_use = optional_u32(approximate_years_in_use, "จำนวนปีที่ใช้มาแล้ว")?;
    if explicit_start.is_some() && prior_use.is_some() {
        return Err(ServerFnError::new(
            "กรอกปีเริ่มใช้หรือจำนวนปีที่ใช้มาแล้วเพียงอย่างเดียว",
        ));
    }
    let start_year = match (explicit_start, prior_use) {
        (Some(year), None) => year,
        (None, Some(years)) => calc::start_year_from_prior_use(
            season_year.ok_or_else(|| ServerFnError::new("ฤดูกาลนี้ยังไม่มีปี พ.ศ."))?,
            years,
        )
        .map_err(|_| ServerFnError::new("จำนวนปีที่ใช้มาแล้วอยู่นอกขอบเขต"))?,
        (None, None) => return Err(ServerFnError::new("กรุณากรอกปีเริ่มใช้หรือจำนวนปีที่ใช้มาแล้ว")),
        (Some(_), Some(_)) => unreachable!(),
    };
    let (useful_life_years, residual_value) = match kind {
        calc::AssetKind::Equipment => (
            Some(required_u32(useful_life_years, "อายุใช้งาน")?),
            optional_money(residual_value, "มูลค่าคงเหลือ", true)?,
        ),
        calc::AssetKind::OwnedLand => {
            if !useful_life_years.trim().is_empty() || !residual_value.trim().is_empty() {
                return Err(ServerFnError::new(
                    "ที่ดินที่เป็นเจ้าของไม่มีอายุใช้งานหรือมูลค่าคงเหลือในแบบจำลองนี้",
                ));
            }
            (None, None)
        }
    };
    let facts = calc::AssetFacts {
        name: name.into(),
        kind,
        original_cost: required_money(original_cost, "ราคาซื้อหรือมูลค่าเดิม")?,
        start_year,
        useful_life_years,
        residual_value,
        retired_year: optional_year(retired_year, "ปีเลิกใช้")?,
    };
    calc::validate(&facts)
        .map_err(|error| ServerFnError::new(format!("ข้อมูลสินทรัพย์ไม่ถูกต้อง: {error:?}")))?;
    Ok(facts)
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn required_money(
    value: &str,
    label: &'static str,
) -> Result<rust_decimal::Decimal, ServerFnError> {
    optional_money(value, label, false)?
        .ok_or_else(|| ServerFnError::new(format!("กรุณากรอก{label}")))
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn optional_money(
    value: &str,
    label: &'static str,
    allow_zero: bool,
) -> Result<Option<rust_decimal::Decimal>, ServerFnError> {
    let normalized = value.trim().replace(',', "");
    if normalized.is_empty() {
        return Ok(None);
    }
    let parsed = normalized
        .parse::<rust_decimal::Decimal>()
        .map_err(|_| ServerFnError::new(format!("{label}ต้องเป็นตัวเลข")))?;
    let invalid_min = if allow_zero {
        parsed < rust_decimal::Decimal::ZERO
    } else {
        parsed <= rust_decimal::Decimal::ZERO
    };
    if invalid_min || parsed > rust_decimal::Decimal::from(calc::MAX_ASSET_VALUE) {
        return Err(ServerFnError::new(format!(
            "{label}อยู่นอกขอบเขตที่ระบบคำนวณได้"
        )));
    }
    Ok(Some(parsed))
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn required_u32(value: &str, label: &'static str) -> Result<u32, ServerFnError> {
    optional_u32(value, label)?.ok_or_else(|| ServerFnError::new(format!("กรุณากรอก{label}")))
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn optional_u32(value: &str, label: &'static str) -> Result<Option<u32>, ServerFnError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let parsed = value
        .parse::<u32>()
        .map_err(|_| ServerFnError::new(format!("{label}ต้องเป็นจำนวนปีเต็ม")))?;
    if parsed > calc::MAX_USEFUL_LIFE_YEARS {
        return Err(ServerFnError::new(format!(
            "{label}ต้องไม่เกิน {} ปี",
            calc::MAX_USEFUL_LIFE_YEARS
        )));
    }
    Ok(Some(parsed))
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn optional_year(value: &str, label: &'static str) -> Result<Option<i32>, ServerFnError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() != 4 || !value.chars().all(|character| character.is_ascii_digit()) {
        return Err(ServerFnError::new(format!("{label}ต้องเป็นปี พ.ศ. 4 หลัก")));
    }
    let parsed = value
        .parse::<i32>()
        .map_err(|_| ServerFnError::new(format!("{label}ไม่ถูกต้อง")))?;
    if !(1000..=9999).contains(&parsed) {
        return Err(ServerFnError::new(format!("{label}อยู่นอกขอบเขต")));
    }
    Ok(Some(parsed))
}

fn asset_kind_label(kind: calc::AssetKind) -> &'static str {
    match kind {
        calc::AssetKind::Equipment => "อุปกรณ์/สิ่งปลูกสร้าง",
        calc::AssetKind::OwnedLand => "ที่ดินที่เป็นเจ้าของ · ไม่คิดค่าเสื่อม",
    }
}

fn decimal_input(value: Option<rust_decimal::Decimal>) -> String {
    value
        .map(|value| value.normalize().to_string())
        .unwrap_or_default()
}

fn money(value: rust_decimal::Decimal) -> String {
    crate::plan_ui::money(value)
}

fn money_or_dash(value: Option<rust_decimal::Decimal>) -> String {
    value.map_or_else(|| "—".into(), |value| format!("{} บาท", money(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_parser_supports_explicit_year_or_prior_use_without_the_clock() {
        let explicit = parse_asset(
            "ปั๊ม",
            "equipment",
            "100000",
            "2568",
            "",
            "5",
            "",
            "",
            Some(2569),
        )
        .unwrap();
        assert_eq!(explicit.start_year, 2568);
        assert_eq!(explicit.residual_value, None);
        let prior = parse_asset(
            "ปั๊ม",
            "equipment",
            "100000",
            "",
            "3",
            "5",
            "0",
            "",
            Some(2570),
        )
        .unwrap();
        assert_eq!(prior.start_year, 2567);
        assert_eq!(prior.residual_value, Some(rust_decimal::Decimal::ZERO));
        assert!(
            parse_asset(
                "ปั๊ม",
                "equipment",
                "100000",
                "2568",
                "3",
                "5",
                "",
                "",
                Some(2570)
            )
            .is_err()
        );
    }

    #[test]
    fn land_rejects_depreciation_fields_and_money_is_bounded() {
        assert!(
            parse_asset(
                "ที่ดิน",
                "owned_land",
                "2000000",
                "2550",
                "",
                "5",
                "",
                "",
                Some(2569)
            )
            .is_err()
        );
        let land = parse_asset(
            "ที่ดิน",
            "owned_land",
            "2000000",
            "2550",
            "",
            "",
            "",
            "",
            Some(2569),
        )
        .unwrap();
        assert_eq!(land.kind, calc::AssetKind::OwnedLand);
        assert_eq!(land.useful_life_years, None);
        assert!(required_money("0", "มูลค่า").is_err());
        assert!(required_money("1000000000001", "มูลค่า").is_err());
    }
}
