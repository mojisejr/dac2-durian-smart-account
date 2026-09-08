use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::plan_form::PlanForm;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanSummary {
    pub id: i64,
    pub name: String,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanRecord {
    pub id: i64,
    pub closed: bool,
    pub form: PlanForm,
}

#[server]
pub async fn list_plans() -> Result<Vec<PlanSummary>, ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    list_for_owner(&pool, owner_id).await
}

#[server]
pub async fn create_sample_plan() -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let created = create_sample_for_owner(&pool, owner_id).await?;
    leptos_axum::redirect(&format!("/plans/{}", created.id));
    Ok(())
}

#[server]
pub async fn create_empty_plan(name: String) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let created = create_empty_for_owner(&pool, owner_id, &name).await?;
    leptos_axum::redirect(&format!("/plans/{}", created.id));
    Ok(())
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
pub async fn clear_plan(id: i64) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    clear_for_owner(&pool, owner_id, id).await?;
    leptos_axum::redirect(&format!("/plans/{id}"));
    Ok(())
}

#[server]
pub async fn duplicate_plan(id: i64, new_name: String) -> Result<(), ServerFnError> {
    let (pool, owner_id) = authenticated_owner().await?;
    let duplicate = duplicate_for_owner(&pool, owner_id, id, &new_name).await?;
    leptos_axum::redirect(&format!("/plans/{}", duplicate.id));
    Ok(())
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
                    closed: summary.closed,
                })
                .collect()
        })
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn create_sample_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
) -> Result<PlanRecord, ServerFnError> {
    store::plans::create(pool, owner_id, &calc::workbook_sample())
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn create_empty_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    name: &str,
) -> Result<PlanRecord, ServerFnError> {
    let mut plan = calc::Plan {
        name: name.trim().to_owned(),
        ..calc::Plan::default()
    };
    if plan.name.is_empty() {
        plan.name = "แผนฤดูใหม่".into();
    }
    store::plans::create(pool, owner_id, &plan)
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
pub async fn clear_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
) -> Result<PlanRecord, ServerFnError> {
    let Some(current) = store::plans::load(pool, owner_id, id)
        .await
        .map_err(public_store_error)?
    else {
        return Err(ServerFnError::new("ไม่พบแผนนี้"));
    };
    let empty = calc::Plan {
        name: current.plan.name,
        ..calc::Plan::default()
    };
    store::plans::save(pool, owner_id, id, &empty)
        .await
        .map(record)
        .map_err(public_store_error)
}

#[cfg(feature = "ssr")]
pub async fn duplicate_for_owner(
    pool: &sqlx::PgPool,
    owner_id: store::users::UserId,
    id: i64,
    new_name: &str,
) -> Result<PlanRecord, ServerFnError> {
    let name = if new_name.trim().is_empty() {
        "สำเนาฤดูกาล"
    } else {
        new_name.trim()
    };
    store::plans::duplicate(pool, owner_id, id, name)
        .await
        .map(record)
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
        closed: stored.closed,
        form: PlanForm::from_plan(&stored.plan),
    }
}

#[cfg(feature = "ssr")]
fn public_store_error(error: store::StoreError) -> ServerFnError {
    match error {
        store::StoreError::NotFound => ServerFnError::new("ไม่พบแผนนี้"),
        store::StoreError::Closed => ServerFnError::new("ฤดูกาลนี้ปิดแล้วและแก้ไขไม่ได้"),
        _ => ServerFnError::new("ระบบยังทำรายการไม่ได้ กรุณาลองอีกครั้ง"),
    }
}
