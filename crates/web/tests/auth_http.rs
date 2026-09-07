#![cfg(feature = "ssr")]

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Method, Request, Response, StatusCode, header},
    middleware,
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
use leptos::server_fn::ServerFn;
use sqlx::PgPool;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, cookie::SameSite};

const PASSWORD: &str = "correct-horse-battery-staple";
const NEW_PASSWORD: &str = "a newly chosen orchard password";

#[derive(Clone)]
struct TestState {
    email: String,
}

async fn seed_session(session: Session) -> StatusCode {
    session
        .insert("pre_login", true)
        .await
        .expect("test session can be seeded");
    StatusCode::NO_CONTENT
}

async fn login(State(state): State<TestState>, mut auth_session: store::AuthSession) -> StatusCode {
    let credentials = store::AuthCredentials {
        email: state.email,
        password: PASSWORD.into(),
    };
    match web::auth::authenticate_and_login(&mut auth_session, credentials).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::UNAUTHORIZED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn logout(mut auth_session: store::AuthSession) -> StatusCode {
    match web::auth::logout_session(&mut auth_session).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn request(method: Method, uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).expect("valid test request")
}

fn form_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .expect("valid form request")
}

async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("response body is readable")
        .to_vec()
}

fn response_cookie(response: &Response<Body>) -> String {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("response sets a session cookie")
        .to_str()
        .expect("cookie is ASCII")
        .split(';')
        .next()
        .expect("cookie has a name and value")
        .to_owned()
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_rotates_session_logout_flushes_it_and_reset_ends_existing_sessions(pool: PgPool) {
    let email = "session@example.test".to_owned();
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(false)
        .with_signed(tower_sessions::cookie::Key::generate());
    let auth_layer =
        AuthManagerLayerBuilder::new(store::AuthBackend::new(pool.clone()), session_layer).build();
    let app = Router::new()
        .route("/seed", get(seed_session))
        .route("/auth/verify-email", get(web::auth::verify_email_link))
        .route("/api/{*fn_name}", post(leptos_axum::handle_server_fns))
        .route("/login-test", post(login))
        .route("/logout-test", post(logout))
        .route("/plans", get(|| async { StatusCode::OK }))
        .route_layer(middleware::from_fn(web::auth::require_login))
        .layer(auth_layer)
        .with_state(TestState {
            email: email.clone(),
        });

    let registration_form = format!("email={email}&password={PASSWORD}");
    let registered = app
        .clone()
        .oneshot(form_request(
            <web::auth::Register as ServerFn>::PATH,
            &registration_form,
        ))
        .await
        .expect("registration request completes");
    assert_eq!(registered.status(), StatusCode::OK);
    let registration_response = body_bytes(registered).await;
    let duplicate = app
        .clone()
        .oneshot(form_request(
            <web::auth::Register as ServerFn>::PATH,
            &registration_form,
        ))
        .await
        .expect("duplicate registration request completes");
    assert_eq!(registration_response, body_bytes(duplicate).await);

    let unverified_login = app
        .clone()
        .oneshot(request(Method::POST, "/login-test", None))
        .await
        .expect("unverified login request completes");
    assert_eq!(unverified_login.status(), StatusCode::UNAUTHORIZED);

    let known_resend = app
        .clone()
        .oneshot(form_request(
            <web::auth::ResendVerification as ServerFn>::PATH,
            &format!("email={email}"),
        ))
        .await
        .expect("known resend request completes");
    let unknown_resend = app
        .clone()
        .oneshot(form_request(
            <web::auth::ResendVerification as ServerFn>::PATH,
            "email=unknown%40example.test",
        ))
        .await
        .expect("unknown resend request completes");
    assert_eq!(
        body_bytes(known_resend).await,
        body_bytes(unknown_resend).await
    );

    let user = store::users::find_by_email(&pool, &email)
        .await
        .expect("account lookup succeeds")
        .expect("registered account exists");
    let verification = store::verification_tokens::issue(&pool, user.id)
        .await
        .expect("verification token issues");

    let verification_uri = format!("/auth/verify-email?token={}", verification.expose());
    let verified = app
        .clone()
        .oneshot(request(Method::GET, &verification_uri, None))
        .await
        .expect("verification link completes");
    assert_eq!(verified.status(), StatusCode::SEE_OTHER);
    assert_eq!(verified.headers()[header::LOCATION], "/login?verified=1");
    let reused = app
        .clone()
        .oneshot(request(Method::GET, &verification_uri, None))
        .await
        .expect("reused verification link is refused");
    assert_eq!(
        reused.headers()[header::LOCATION],
        "/verify-email?invalid=1"
    );

    let known_reset = app
        .clone()
        .oneshot(form_request(
            <web::auth::RequestPasswordReset as ServerFn>::PATH,
            &format!("email={email}"),
        ))
        .await
        .expect("known reset request completes");
    let unknown_reset = app
        .clone()
        .oneshot(form_request(
            <web::auth::RequestPasswordReset as ServerFn>::PATH,
            "email=unknown%40example.test",
        ))
        .await
        .expect("unknown reset request completes");
    assert_eq!(
        body_bytes(known_reset).await,
        body_bytes(unknown_reset).await
    );

    let rejected = app
        .clone()
        .oneshot(request(Method::GET, "/plans", None))
        .await
        .expect("request completes");
    assert_eq!(rejected.status(), StatusCode::SEE_OTHER);

    let seeded = app
        .clone()
        .oneshot(request(Method::GET, "/seed", None))
        .await
        .expect("session seed completes");
    let cookie_before_login = response_cookie(&seeded);
    let cookie_attributes = seeded.headers()[header::SET_COOKIE]
        .to_str()
        .expect("cookie is ASCII");
    assert!(cookie_attributes.contains("HttpOnly"));
    assert!(cookie_attributes.contains("SameSite=Lax"));

    let logged_in = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/login-test",
            Some(&cookie_before_login),
        ))
        .await
        .expect("login completes");
    assert_eq!(logged_in.status(), StatusCode::NO_CONTENT);
    let cookie_after_login = response_cookie(&logged_in);
    assert_ne!(cookie_before_login, cookie_after_login);

    let old_session = app
        .clone()
        .oneshot(request(Method::GET, "/plans", Some(&cookie_before_login)))
        .await
        .expect("old session request completes");
    assert_eq!(old_session.status(), StatusCode::SEE_OTHER);
    let accepted = app
        .clone()
        .oneshot(request(Method::GET, "/plans", Some(&cookie_after_login)))
        .await
        .expect("authenticated request completes");
    assert_eq!(accepted.status(), StatusCode::OK);

    let logged_out = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/logout-test",
            Some(&cookie_after_login),
        ))
        .await
        .expect("logout completes");
    assert_eq!(logged_out.status(), StatusCode::NO_CONTENT);
    let after_logout = app
        .clone()
        .oneshot(request(Method::GET, "/plans", Some(&cookie_after_login)))
        .await
        .expect("logged-out request completes");
    assert_eq!(after_logout.status(), StatusCode::SEE_OTHER);

    let seeded_again = app
        .clone()
        .oneshot(request(Method::GET, "/seed", None))
        .await
        .expect("second session seed completes");
    let second_seed_cookie = response_cookie(&seeded_again);
    let logged_in_again = app
        .clone()
        .oneshot(request(
            Method::POST,
            "/login-test",
            Some(&second_seed_cookie),
        ))
        .await
        .expect("second login completes");
    let second_login_cookie = response_cookie(&logged_in_again);

    let reset = store::reset_tokens::issue(&pool, user.id)
        .await
        .expect("reset token issues");
    store::reset_tokens::consume(&pool, reset.expose(), NEW_PASSWORD)
        .await
        .expect("password resets");
    let after_reset = app
        .oneshot(request(Method::GET, "/plans", Some(&second_login_cookie)))
        .await
        .expect("post-reset request completes");
    assert_eq!(after_reset.status(), StatusCode::SEE_OTHER);
}
