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
    pub form: PlanForm,
}

#[server]
pub async fn list_plans() -> Result<Vec<PlanSummary>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    list_for_owner(&pool, owner_id).await
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
    leptos_axum::redirect(&format!("/plans/{}", created.id));
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
pub async fn close_plan(id: i64) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    close_for_owner(&pool, owner_id, id).await?;
    leptos_axum::redirect(&format!("/plans/{id}"));
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
    store::plans::create(pool, owner_id, season_year, note, &plan)
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
pub async fn close_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
) -> Result<(), ServerFnError> {
    store::plans::close(pool, owner_id, id)
        .await
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
fn record(stored: store::plans::StoredPlan) -> PlanRecord {
    PlanRecord {
        id: stored.id,
        season_year: stored.season_year,
        note: stored.note,
        closed: stored.closed,
        form: PlanForm::from_plan(&stored.plan),
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
}
