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

    let created = match web::plans::create_empty_for_owner(pool, user.id, "ฤดูทดสอบ").await
    {
        Ok(record) => record,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
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

    let duplicate = match web::plans::duplicate_for_owner(pool, user.id, created.id, "ฤดูถัดไป").await
    {
        Ok(record) => record,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if duplicate.closed || duplicate.form.name != "ฤดูถัดไป" {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let cleared = match web::plans::clear_for_owner(pool, user.id, duplicate.id).await {
        Ok(record) => record,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let cleared_plan = match cleared.form.to_plan() {
        Ok(plan) => plan,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if cleared_plan.name != "ฤดูถัดไป"
        || cleared_plan.market != calc::MarketPlan::default()
        || !cleared_plan.production.grades.is_empty()
        || !cleared_plan.variable_costs.is_empty()
        || !cleared_plan.fixed_costs.is_empty()
        || !cleared_plan.health_answers.is_empty()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if web::plans::close_for_owner(pool, user.id, created.id)
        .await
        .is_err()
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
