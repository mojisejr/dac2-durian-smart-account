#![cfg(feature = "ssr")]

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
use calc::Grade;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, cookie::SameSite};
use web::plan_form::PlanForm;

const EMAIL: &str = "plan-owner@example.test";
const PASSWORD: &str = "correct-horse-battery-staple";

async fn login(mut auth_session: store::AuthSession) -> StatusCode {
    let credentials = store::AuthCredentials {
        email: EMAIL.into(),
        password: PASSWORD.into(),
    };
    match web::auth::authenticate_and_login(&mut auth_session, credentials).await {
        Ok(true) => StatusCode::NO_CONTENT,
        _ => StatusCode::UNAUTHORIZED,
    }
}

async fn plan_flow(auth_session: store::AuthSession) -> StatusCode {
    let Some(user) = auth_session.user else {
        return StatusCode::UNAUTHORIZED;
    };
    let pool = auth_session.backend.pool();

    let created =
        match web::plans::create_empty_for_owner(pool, user.id, 2569, "ฤดูทดสอบ", "แผนรวมทุกแปลง")
            .await
        {
            Ok(record) => record,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
    if created.forecast_mode != calc::ForecastMode::Quick
        || created.quick_estimate != calc::QuickEstimate::default()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    for (field, value) in [
        (
            calc::QuickInputField::SellableYieldKg,
            Decimal::from(20_000),
        ),
        (calc::QuickInputField::AveragePricePerKg, Decimal::from(80)),
        (calc::QuickInputField::TotalCost, Decimal::from(900_000)),
    ] {
        if web::plans::save_quick_value_for_owner(pool, user.id, created.id, field, value)
            .await
            .is_err()
        {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    let mut expected = calc::workbook_sample();
    expected.name = "ฤดูทดสอบฉบับเต็ม".into();
    expected.production.grades = (1..=10)
        .map(|index| Grade {
            name: format!("เกรด {index}"),
            share: Some(Decimal::new(1, 1)),
            price_per_kg: Some(Decimal::from(index * 10)),
            counts_as_quality_grade: index <= 2,
        })
        .collect();
    let form = PlanForm::from_plan(&expected);
    if web::plans::save_for_owner(pool, user.id, created.id, &form)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let loaded = match web::plans::load_for_owner(pool, user.id, created.id).await {
        Ok(Some(record)) => record,
        _ => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if loaded.form.to_plan().ok().as_ref() != Some(&expected) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let quick = calc::analyze_quick(&loaded.quick_estimate);
    if quick.net_profit != Some(Decimal::from(700_000)) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let detailed = match web::plans::set_forecast_mode_for_owner(
        pool,
        user.id,
        created.id,
        calc::ForecastMode::Detailed,
    )
    .await
    {
        Ok(record) => record,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if detailed.form.to_plan().ok().as_ref() != Some(&expected)
        || detailed.quick_estimate != loaded.quick_estimate
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let duplicate =
        match web::plans::duplicate_for_owner(pool, user.id, created.id, 2570, "ฤดูถัดไป", "").await
        {
            Ok(record) => record,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
    if duplicate.closed
        || duplicate.form.name != "ฤดูถัดไป"
        || duplicate.season_year != Some(2570)
        || duplicate.forecast_mode != calc::ForecastMode::Quick
        || duplicate.quick_estimate != loaded.quick_estimate
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let actual = calc::ActualOutcome {
        sellable_yield_kg: Some(Decimal::from(18_000)),
        revenue: Some(Decimal::from(1_530_000)),
        total_cost: Some(Decimal::from(990_000)),
        note: "ผลผลิตน้อยกว่าคาด".into(),
    };
    if web::plans::save_actual_draft_for_owner(pool, user.id, created.id, &actual)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let finalized = match web::plans::finalize_actual_for_owner(pool, user.id, created.id).await {
        Ok(record) => record,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if !finalized.closed
        || !finalized
            .actual_outcome
            .as_ref()
            .is_some_and(|outcome| outcome.finalized)
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if web::plans::finalize_actual_for_owner(pool, user.id, created.id)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let history = match web::plans::history_for_owner(pool, user.id).await {
        Ok(history) => history,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if history.len() != 1
        || history[0].id != created.id
        || history[0]
            .actual_outcome
            .as_ref()
            .and_then(|actual| actual.forecast.as_ref())
            .and_then(|forecast| forecast.profit)
            .is_none()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if web::plans::save_for_owner(pool, user.id, created.id, &form)
        .await
        .is_ok()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::NO_CONTENT
}

fn request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut request = Request::builder().method(Method::POST).uri(uri);
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    request.body(Body::empty()).expect("valid request")
}

fn response_cookie(response: &axum::response::Response) -> String {
    response.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is ASCII")
        .split(';')
        .next()
        .expect("cookie has a value")
        .to_owned()
}

#[sqlx::test(migrations = "../../migrations")]
async fn authenticated_plan_operations_save_reload_duplicate_and_close(pool: PgPool) {
    let user = store::users::register(&pool, EMAIL, PASSWORD)
        .await
        .expect("registration succeeds");
    sqlx::query("UPDATE users SET email_verified_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("test owner is verified");

    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(false)
        .with_signed(tower_sessions::cookie::Key::generate());
    let auth_layer =
        AuthManagerLayerBuilder::new(store::AuthBackend::new(pool), session_layer).build();
    let app = Router::new()
        .route("/login", post(login))
        .route("/plan-flow", post(plan_flow))
        .route("/plans", get(|| async { StatusCode::OK }))
        .layer(auth_layer);

    let unauthenticated = app
        .clone()
        .oneshot(request("/plan-flow", None))
        .await
        .expect("request completes");
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let logged_in = app
        .clone()
        .oneshot(request("/login", None))
        .await
        .expect("login completes");
    assert_eq!(logged_in.status(), StatusCode::NO_CONTENT);
    let cookie = response_cookie(&logged_in);

    let result = app
        .oneshot(request("/plan-flow", Some(&cookie)))
        .await
        .expect("plan flow completes");
    assert_eq!(result.status(), StatusCode::NO_CONTENT);
}
