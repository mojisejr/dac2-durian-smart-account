mod codec;
mod read;
mod types;
mod write;

pub use types::{PlanId, PlanSummary, StoreError, StoredPlan};

use calc::Plan;
use sqlx::{PgConnection, PgPool, Row};

use crate::users::UserId;

pub async fn list(pool: &PgPool, owner_id: UserId) -> Result<Vec<PlanSummary>, StoreError> {
    let rows = sqlx::query(
        "SELECT id, name, closed_at IS NOT NULL AS closed FROM plans \
         WHERE owner_id = $1 ORDER BY id DESC",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(PlanSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                closed: row.try_get("closed")?,
            })
        })
        .collect()
}

pub async fn create(
    pool: &PgPool,
    owner_id: UserId,
    plan: &Plan,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    let id: PlanId =
        sqlx::query_scalar("INSERT INTO plans (owner_id, name) VALUES ($1, $2) RETURNING id")
            .bind(owner_id)
            .bind(&plan.name)
            .fetch_one(transaction.as_mut())
            .await?;
    write::replace_sections(transaction.as_mut(), owner_id, id, plan).await?;
    transaction.commit().await?;

    Ok(StoredPlan {
        id,
        owner_id,
        closed: false,
        plan: plan.clone(),
    })
}

pub async fn load(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
) -> Result<Option<StoredPlan>, StoreError> {
    let mut connection = pool.acquire().await?;
    read::load(&mut connection, owner_id, id).await
}

pub async fn save(
    pool: &PgPool,
    owner_id: UserId,
    id: PlanId,
    plan: &Plan,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("UPDATE plans SET name = $3 WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .bind(&plan.name)
        .execute(transaction.as_mut())
        .await?;
    write::replace_sections(transaction.as_mut(), owner_id, id, plan).await?;
    transaction.commit().await?;

    Ok(StoredPlan {
        id,
        owner_id,
        closed: false,
        plan: plan.clone(),
    })
}

pub async fn close(pool: &PgPool, owner_id: UserId, id: PlanId) -> Result<(), StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("UPDATE plans SET closed_at = CURRENT_TIMESTAMP WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, owner_id: UserId, id: PlanId) -> Result<(), StoreError> {
    let mut transaction = pool.begin().await?;
    ensure_open(transaction.as_mut(), owner_id, id).await?;
    sqlx::query("DELETE FROM plans WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(transaction.as_mut())
        .await?;
    transaction.commit().await?;
    Ok(())
}

/// Deep-copy a source plan without mutating it. A closed season may therefore
/// be used as the source of a new open season while remaining read-only itself.
pub async fn duplicate(
    pool: &PgPool,
    owner_id: UserId,
    source_id: PlanId,
    new_name: &str,
) -> Result<StoredPlan, StoreError> {
    let mut transaction = pool.begin().await?;
    let Some(mut source) = read::load(transaction.as_mut(), owner_id, source_id).await? else {
        return Err(StoreError::NotFound);
    };
    source.plan.name = new_name.into();
    let id: PlanId =
        sqlx::query_scalar("INSERT INTO plans (owner_id, name) VALUES ($1, $2) RETURNING id")
            .bind(owner_id)
            .bind(&source.plan.name)
            .fetch_one(transaction.as_mut())
            .await?;
    write::replace_sections(transaction.as_mut(), owner_id, id, &source.plan).await?;
    transaction.commit().await?;

    Ok(StoredPlan {
        id,
        owner_id,
        closed: false,
        plan: source.plan,
    })
}

async fn ensure_open(
    connection: &mut PgConnection,
    owner_id: UserId,
    id: PlanId,
) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT closed_at IS NOT NULL AS closed FROM plans \
         WHERE id = $1 AND owner_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(connection)
    .await?;
    let Some(row) = row else {
        return Err(StoreError::NotFound);
    };
    if row.try_get::<bool, _>("closed")? {
        return Err(StoreError::Closed);
    }
    Ok(())
}
