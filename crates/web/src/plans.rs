use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::plan_form::PlanForm;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: i64,
    pub name: String,
    pub season_year: Option<i32>,
    pub note: String,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanRecord {
    pub id: i64,
    pub season_year: Option<i32>,
    pub note: String,
    pub closed: bool,
    pub forecast_mode: calc::ForecastMode,
    pub quick_estimate: calc::QuickEstimate,
    pub actual_outcome: Option<ActualOutcomeRecord>,
    pub form: PlanForm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActualOutcomeRecord {
    pub outcome: calc::ActualOutcome,
    pub finalized: bool,
    pub forecast_mode: Option<calc::ForecastMode>,
    pub forecast: Option<calc::OutcomeMetrics>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonHistoryItem {
    pub id: i64,
    pub name: String,
    pub season_year: Option<i32>,
    pub actual_outcome: Option<ActualOutcomeRecord>,
}

#[server]
pub async fn list_plans() -> Result<Vec<PlanSummary>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    list_for_owner(&pool, owner_id).await
}

#[server]
pub async fn load_season_history() -> Result<Vec<SeasonHistoryItem>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    history_for_owner(&pool, owner_id).await
}

#[server]
pub async fn create_season(
    season_year: String,
    name: String,
    note: String,
    source_id: Option<i64>,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let year = valid_season_year(&season_year)?;
    let name = valid_name(&name)?;
    let note = valid_note(&note)?;
    let created = match source_id {
        Some(id) => duplicate_for_owner(&pool, owner_id, id, year, name, note).await?,
        None => create_empty_for_owner(&pool, owner_id, year, name, note).await?,
    };
    leptos_axum::redirect(&format!("/plans/{}/quick/production", created.id));
    Ok(())
}

#[server]
pub async fn update_season_metadata(
    id: i64,
    season_year: String,
    name: String,
    note: String,
) -> Result<String, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let year = valid_season_year(&season_year)?;
    let name = valid_name(&name)?;
    let note = valid_note(&note)?;
    update_metadata_for_owner(&pool, owner_id, id, year, name, note).await?;
    Ok("บันทึกรายละเอียดฤดูกาลแล้ว".into())
}

#[server]
pub async fn load_plan(id: i64) -> Result<Option<PlanRecord>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    load_for_owner(&pool, owner_id, id).await
}

#[server]
pub async fn save_plan(id: i64, form_json: String) -> Result<String, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let form: PlanForm =
        serde_json::from_str(&form_json).map_err(|_| ServerFnError::new("ข้อมูลแบบฟอร์มไม่ถูกต้อง"))?;
    save_for_owner(&pool, owner_id, id, &form).await?;
    Ok("บันทึกแล้ว".into())
}

#[server]
pub async fn save_quick_step(id: i64, step: String, value: String) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let field = quick_field(&step)?;
    let value = valid_quick_value(&value, field)?;
    save_quick_value_for_owner(&pool, owner_id, id, field, value).await?;
    leptos_axum::redirect(&quick_next_path(id, field));
    Ok(())
}

#[server]
pub async fn switch_forecast_mode(id: i64, mode: String) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let mode = match mode.as_str() {
        "quick" => calc::ForecastMode::Quick,
        "detailed" => calc::ForecastMode::Detailed,
        _ => return Err(ServerFnError::new("โหมดประมาณการไม่ถูกต้อง")),
    };
    let record = set_forecast_mode_for_owner(&pool, owner_id, id, mode).await?;
    let destination = match mode {
        calc::ForecastMode::Quick => quick_resume_path(id, &record.quick_estimate),
        calc::ForecastMode::Detailed => format!("/plans/{id}"),
    };
    leptos_axum::redirect(&destination);
    Ok(())
}

#[server]
pub async fn save_actual_draft(
    id: i64,
    sellable_yield_kg: String,
    revenue: String,
    total_cost: String,
    note: String,
) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let outcome = valid_actual_outcome(&sellable_yield_kg, &revenue, &total_cost, &note)?;
    save_actual_draft_for_owner(&pool, owner_id, id, &outcome).await?;
    leptos_axum::redirect(&format!("/plans/{id}/close/review"));
    Ok(())
}

#[server]
pub async fn finalize_actual(id: i64) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    finalize_actual_for_owner(&pool, owner_id, id).await?;
    leptos_axum::redirect(&format!("/plans/{id}/comparison"));
    Ok(())
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
pub async fn list_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
) -> Result<Vec<PlanSummary>, ServerFnError> {
    store::plans::list(pool, owner_id)
        .await
        .map(|summaries| {
            summaries
                .into_iter()
                .map(|summary| PlanSummary {
                    id: summary.id,
                    name: summary.name,
                    season_year: summary.season_year,
                    note: summary.note,
                    closed: summary.closed,
                })
                .collect()
        })
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn history_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
) -> Result<Vec<SeasonHistoryItem>, ServerFnError> {
    store::plans::history(pool, owner_id)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| SeasonHistoryItem {
                    id: row.id,
                    name: row.name,
                    season_year: row.season_year,
                    actual_outcome: row.actual_outcome.map(actual_record),
                })
                .collect()
        })
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn create_empty_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    season_year: i32,
    name: &str,
    note: &str,
) -> Result<PlanRecord, ServerFnError> {
    let plan = calc::Plan {
        name: name.trim().to_owned(),
        ..calc::Plan::default()
    };
    store::plans::create_quick(pool, owner_id, season_year, note, &plan)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn load_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
) -> Result<Option<PlanRecord>, ServerFnError> {
    store::plans::load(pool, owner_id, id)
        .await
        .map(|record| record.map(self::record))
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn save_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    form: &PlanForm,
) -> Result<PlanRecord, ServerFnError> {
    let plan = form.to_plan().map_err(|errors| {
        let message = errors
            .first()
            .map(|error| format!("{}: {}", error.field, error.message))
            .unwrap_or_else(|| "ข้อมูลยังไม่ถูกต้อง".into());
        ServerFnError::new(message)
    })?;
    store::plans::save(pool, owner_id, id, &plan)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn save_quick_value_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    field: calc::QuickInputField,
    value: rust_decimal::Decimal,
) -> Result<PlanRecord, ServerFnError> {
    let Some(stored) = store::plans::load(pool, owner_id, id)
        .await
        .map_err(public_store_error)?
    else {
        return Err(ServerFnError::new("ไม่พบฤดูกาลนี้"));
    };
    let mut estimate = stored.quick_estimate;
    match field {
        calc::QuickInputField::SellableYieldKg => estimate.sellable_yield_kg = Some(value),
        calc::QuickInputField::AveragePricePerKg => {
            estimate.average_price_per_kg = Some(value);
        }
        calc::QuickInputField::TotalCost => estimate.total_cost = Some(value),
    }
    store::plans::save_quick(pool, owner_id, id, &estimate)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn set_forecast_mode_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    mode: calc::ForecastMode,
) -> Result<PlanRecord, ServerFnError> {
    store::plans::set_forecast_mode(pool, owner_id, id, mode)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn duplicate_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    season_year: i32,
    new_name: &str,
    note: &str,
) -> Result<PlanRecord, ServerFnError> {
    store::plans::duplicate(pool, owner_id, id, season_year, new_name.trim(), note)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn update_metadata_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    season_year: i32,
    name: &str,
    note: &str,
) -> Result<(), ServerFnError> {
    store::plans::update_metadata(pool, owner_id, id, season_year, name, note)
        .await
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn save_actual_draft_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    outcome: &calc::ActualOutcome,
) -> Result<PlanRecord, ServerFnError> {
    store::plans::save_actual_draft(pool, owner_id, id, outcome)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn finalize_actual_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
) -> Result<PlanRecord, ServerFnError> {
    let stored = store::plans::load(pool, owner_id, id)
        .await
        .map_err(public_store_error)?
        .ok_or_else(|| ServerFnError::new("ไม่พบฤดูกาลนี้"))?;
    let outcome = stored
        .actual_outcome
        .as_ref()
        .map(|actual| actual.outcome.clone())
        .ok_or_else(|| ServerFnError::new("กรุณากรอกผลจริงก่อนตรวจทานและปิดฤดูกาล"))?;
    store::plans::finalize_with_actual(pool, owner_id, id, &outcome)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
fn record(stored: store::plans::StoredPlan) -> PlanRecord {
    PlanRecord {
        id: stored.id,
        season_year: stored.season_year,
        note: stored.note,
        closed: stored.closed,
        forecast_mode: stored.forecast_mode,
        quick_estimate: stored.quick_estimate,
        actual_outcome: stored.actual_outcome.map(actual_record),
        form: PlanForm::from_plan(&stored.plan),
    }
}

#[cfg(feature = "ssr")]
fn actual_record(actual: store::plans::StoredActualOutcome) -> ActualOutcomeRecord {
    ActualOutcomeRecord {
        outcome: actual.outcome,
        finalized: actual.finalized,
        forecast_mode: actual.forecast_mode,
        forecast: actual.forecast,
    }
}

#[cfg(feature = "ssr")]
fn public_store_error(error: store::StoreError) -> ServerFnError {
    match error {
        store::StoreError::NotFound => ServerFnError::new("ไม่พบฤดูกาลนี้"),
        store::StoreError::Closed => ServerFnError::new("ฤดูกาลนี้ปิดแล้วและแก้ไขไม่ได้"),
        store::StoreError::DuplicateSeasonYear => {
            ServerFnError::new("มีฤดูกาลสำหรับปีนี้แล้ว กรุณาเลือกปีอื่น")
        }
        _ => ServerFnError::new("ระบบยังทำรายการไม่ได้ กรุณาลองอีกครั้ง"),
    }
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_season_year(value: &str) -> Result<i32, ServerFnError> {
    let trimmed = value.trim();
    if trimmed.len() != 4 || !trimmed.chars().all(|character| character.is_ascii_digit()) {
        return Err(ServerFnError::new("กรุณากรอกปี พ.ศ. เป็นตัวเลข 4 หลัก"));
    }
    trimmed
        .parse()
        .map_err(|_| ServerFnError::new("กรุณากรอกปี พ.ศ. เป็นตัวเลข 4 หลัก"))
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_name(value: &str) -> Result<&str, ServerFnError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ServerFnError::new("กรุณาตั้งชื่อฤดูกาล"));
    }
    if value.chars().count() > 120 {
        return Err(ServerFnError::new("ชื่อฤดูกาลยาวเกิน 120 ตัวอักษร"));
    }
    Ok(value)
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_note(value: &str) -> Result<&str, ServerFnError> {
    let value = value.trim();
    if value.chars().count() > 2_000 {
        return Err(ServerFnError::new("บันทึกยาวเกิน 2,000 ตัวอักษร"));
    }
    Ok(value)
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_actual_outcome(
    sellable_yield_kg: &str,
    revenue: &str,
    total_cost: &str,
    note: &str,
) -> Result<calc::ActualOutcome, ServerFnError> {
    Ok(calc::ActualOutcome {
        sellable_yield_kg: Some(valid_actual_value(sellable_yield_kg, "ผลผลิตที่ขายได้จริง")?),
        revenue: Some(valid_actual_value(revenue, "รายได้จริง")?),
        total_cost: Some(valid_actual_value(total_cost, "ต้นทุนรวมจริง")?),
        note: valid_note(note)?.into(),
    })
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_actual_value(
    value: &str,
    label: &'static str,
) -> Result<rust_decimal::Decimal, ServerFnError> {
    use std::str::FromStr;

    let normalized = value.trim().replace(',', "");
    if normalized.is_empty() {
        return Err(ServerFnError::new(format!("กรุณากรอก{label}")));
    }
    let parsed = rust_decimal::Decimal::from_str(&normalized)
        .map_err(|_| ServerFnError::new(format!("{label}ต้องเป็นตัวเลข")))?;
    if parsed < rust_decimal::Decimal::ZERO {
        return Err(ServerFnError::new(format!("{label}ต้องไม่ติดลบ")));
    }
    let maximum = rust_decimal::Decimal::from(1_000_000_000_000_i64);
    if parsed > maximum {
        return Err(ServerFnError::new(format!(
            "{label}สูงเกินขอบเขตที่ระบบคำนวณได้"
        )));
    }
    Ok(parsed)
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn quick_field(value: &str) -> Result<calc::QuickInputField, ServerFnError> {
    match value {
        "production" => Ok(calc::QuickInputField::SellableYieldKg),
        "price" => Ok(calc::QuickInputField::AveragePricePerKg),
        "cost" => Ok(calc::QuickInputField::TotalCost),
        _ => Err(ServerFnError::new("ไม่พบขั้นตอนประมาณการนี้")),
    }
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn quick_field_label(field: calc::QuickInputField) -> &'static str {
    match field {
        calc::QuickInputField::SellableYieldKg => "ผลผลิตที่ขายได้",
        calc::QuickInputField::AveragePricePerKg => "ราคาขายเฉลี่ย",
        calc::QuickInputField::TotalCost => "ต้นทุนรวม",
    }
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn valid_quick_value(
    value: &str,
    field: calc::QuickInputField,
) -> Result<rust_decimal::Decimal, ServerFnError> {
    use std::str::FromStr;

    let label = quick_field_label(field);
    let normalized = value.trim().replace(',', "");
    if normalized.is_empty() {
        return Err(ServerFnError::new(format!("กรุณากรอก{label}")));
    }
    let parsed = rust_decimal::Decimal::from_str(&normalized)
        .map_err(|_| ServerFnError::new(format!("{label}ต้องเป็นตัวเลข")))?;
    if parsed <= rust_decimal::Decimal::ZERO {
        return Err(ServerFnError::new(format!("{label}ต้องมากกว่า 0")));
    }
    let maximum = rust_decimal::Decimal::from(1_000_000_000_000_i64);
    if parsed > maximum {
        return Err(ServerFnError::new(format!(
            "{label}สูงเกินขอบเขตที่ระบบคำนวณได้"
        )));
    }
    Ok(parsed)
}

pub fn quick_resume_path(id: i64, estimate: &calc::QuickEstimate) -> String {
    match estimate.first_incomplete() {
        Some(calc::QuickInputField::SellableYieldKg) => {
            format!("/plans/{id}/quick/production")
        }
        Some(calc::QuickInputField::AveragePricePerKg) => format!("/plans/{id}/quick/price"),
        Some(calc::QuickInputField::TotalCost) => format!("/plans/{id}/quick/cost"),
        None => format!("/plans/{id}/quick/result"),
    }
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn quick_next_path(id: i64, field: calc::QuickInputField) -> String {
    match field {
        calc::QuickInputField::SellableYieldKg => format!("/plans/{id}/quick/price"),
        calc::QuickInputField::AveragePricePerKg => format!("/plans/{id}/quick/cost"),
        calc::QuickInputField::TotalCost => format!("/plans/{id}/quick/result"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_metadata_requires_a_four_digit_year_and_a_name() {
        assert_eq!(valid_season_year("2569").expect("valid year"), 2569);
        assert!(valid_season_year("69").is_err());
        assert!(valid_season_year("๒๕๖๙").is_err());
        assert_eq!(valid_name(" สวนรวม ").expect("valid name"), "สวนรวม");
        assert!(valid_name("   ").is_err());
    }

    #[test]
    fn season_note_is_trimmed_and_bounded() {
        assert_eq!(valid_note(" บันทึก ").expect("valid note"), "บันทึก");
        assert!(valid_note(&"ก".repeat(2_001)).is_err());
    }

    #[test]
    fn quick_values_are_positive_bounded_decimals_with_specific_errors() {
        use calc::QuickInputField::{AveragePricePerKg, SellableYieldKg, TotalCost};

        assert_eq!(
            valid_quick_value(" 20,000.50 ", SellableYieldKg).expect("valid yield"),
            rust_decimal::Decimal::new(2_000_050, 2)
        );
        assert!(
            valid_quick_value("", SellableYieldKg)
                .expect_err("blank fails")
                .to_string()
                .contains("กรุณากรอกผลผลิตที่ขายได้")
        );
        assert!(
            valid_quick_value("ศูนย์", AveragePricePerKg)
                .expect_err("text fails")
                .to_string()
                .contains("ราคาขายเฉลี่ยต้องเป็นตัวเลข")
        );
        assert!(
            valid_quick_value("0", TotalCost)
                .expect_err("zero fails")
                .to_string()
                .contains("ต้นทุนรวมต้องมากกว่า 0")
        );
        assert!(
            valid_quick_value("1000000000001", TotalCost)
                .expect_err("technical upper bound fails")
                .to_string()
                .contains("สูงเกินขอบเขต")
        );
    }

    #[test]
    fn actual_values_accept_real_zeroes_but_reject_missing_negative_and_extreme_values() {
        assert_eq!(
            valid_actual_value("0", "ผลผลิตจริง").expect("a zero-yield season is real"),
            rust_decimal::Decimal::ZERO
        );
        assert!(
            valid_actual_value("", "ผลผลิตจริง")
                .expect_err("blank fails")
                .to_string()
                .contains("กรุณากรอกผลผลิตจริง")
        );
        assert!(
            valid_actual_value("-1", "รายได้จริง")
                .expect_err("negative fails")
                .to_string()
                .contains("รายได้จริงต้องไม่ติดลบ")
        );
        assert!(valid_actual_value("1000000000001", "ต้นทุนจริง").is_err());
        assert!(valid_actual_outcome("1", "2", "3", &"ก".repeat(2_001)).is_err());
    }

    #[test]
    fn quick_resume_uses_the_first_missing_answer_and_then_the_result() {
        let mut estimate = calc::QuickEstimate::default();
        assert_eq!(
            quick_resume_path(42, &estimate),
            "/plans/42/quick/production"
        );
        estimate.sellable_yield_kg = Some(rust_decimal::Decimal::ONE);
        assert_eq!(quick_resume_path(42, &estimate), "/plans/42/quick/price");
        estimate.average_price_per_kg = Some(rust_decimal::Decimal::ONE);
        assert_eq!(quick_resume_path(42, &estimate), "/plans/42/quick/cost");
        estimate.total_cost = Some(rust_decimal::Decimal::ONE);
        assert_eq!(quick_resume_path(42, &estimate), "/plans/42/quick/result");
    }
}
