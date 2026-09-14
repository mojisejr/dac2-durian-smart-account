#![cfg(feature = "ssr")]

//! The network gate over HTTP: the guarded doors refuse after their budget
//! with a wait the client can read, in the shape each kind of client expects,
//! while the rest of the site is untouched. The budget arithmetic itself is
//! covered by the unit tests in `web::gate`; this file proves the wiring.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Method, Request, Response, StatusCode, header},
    middleware,
    routing::{get, post},
};
use leptos::server_fn::ServerFn;
use tower::ServiceExt;
use web::gate::{AddressSource, GateConfig, Limit, Limiter, limit_requests};

fn gated_app(config: GateConfig) -> Router {
    let limiter = Arc::new(Limiter::new(config));
    Router::new()
        .route("/auth/verify-email", get(|| async { StatusCode::OK }))
        .route("/api/{*fn_name}", post(|| async { StatusCode::OK }))
        .route("/plans", get(|| async { StatusCode::OK }))
        .layer(middleware::from_fn_with_state(limiter, limit_requests))
}

fn tight() -> GateConfig {
    GateConfig {
        account: Limit {
            attempts: 2,
            window: Duration::from_secs(60),
        },
        login: Limit {
            attempts: 3,
            window: Duration::from_secs(60),
        },
        address_source: AddressSource::Peer,
    }
}

fn request(method: Method, uri: &str, peer: [u8; 4], accept: &str) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ACCEPT, accept)
        .body(Body::empty())
        .expect("valid test request");
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from((peer, 51000))));
    request
}

async fn text(response: Response<Body>) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body is readable")
            .to_vec(),
    )
    .expect("body is UTF-8")
}

#[tokio::test]
async fn the_account_door_refuses_after_its_budget_with_a_readable_wait() {
    let app = gated_app(tight());
    let register = <web::auth::Register as ServerFn>::PATH;

    for _ in 0..2 {
        let allowed = app
            .clone()
            .oneshot(request(
                Method::POST,
                register,
                [203, 0, 113, 9],
                "text/html",
            ))
            .await
            .expect("request completes");
        assert_eq!(allowed.status(), StatusCode::OK);
    }

    let refused = app
        .clone()
        .oneshot(request(
            Method::POST,
            register,
            [203, 0, 113, 9],
            "text/html",
        ))
        .await
        .expect("request completes");
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u64 = refused
        .headers()
        .get(header::RETRY_AFTER)
        .expect("refusal names how long to wait")
        .to_str()
        .unwrap()
        .parse()
        .expect("Retry-After is whole seconds");
    assert!((1..=60).contains(&retry_after), "{retry_after}");
    assert!(
        refused
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    let page = text(refused).await;
    assert!(page.contains("ลองหลายครั้งติดกันแล้ว กรุณารอ"), "{page}");
    // Plain wording leads; the formal term follows in parentheses.
    let plain = page.find("รอสักครู่").expect("plain heading");
    let term = page.find("rate limit").expect("formal term");
    assert!(plain < term);

    // The other budget and another address are untouched.
    let login = app
        .clone()
        .oneshot(request(
            Method::POST,
            <web::auth::Login as ServerFn>::PATH,
            [203, 0, 113, 9],
            "text/html",
        ))
        .await
        .expect("request completes");
    assert_eq!(login.status(), StatusCode::OK);
    let neighbour = app
        .clone()
        .oneshot(request(
            Method::POST,
            register,
            [203, 0, 113, 10],
            "text/html",
        ))
        .await
        .expect("request completes");
    assert_eq!(neighbour.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_hydrated_form_receives_the_refusal_as_a_server_function_error() {
    let app = gated_app(tight());
    let login = <web::auth::Login as ServerFn>::PATH;
    for _ in 0..3 {
        app.clone()
            .oneshot(request(
                Method::POST,
                login,
                [198, 51, 100, 4],
                "application/json",
            ))
            .await
            .expect("request completes");
    }
    let refused = app
        .clone()
        .oneshot(request(
            Method::POST,
            login,
            [198, 51, 100, 4],
            "application/json",
        ))
        .await
        .expect("request completes");
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = text(refused).await;
    assert!(body.starts_with("ServerError|ลองหลายครั้งติดกันแล้ว"), "{body}");
    assert!(!body.contains('<'), "no markup reaches the form: {body}");
}

#[tokio::test]
async fn the_verification_link_shares_the_login_budget_and_the_site_is_unlimited() {
    let app = gated_app(tight());
    for _ in 0..3 {
        let allowed = app
            .clone()
            .oneshot(request(
                Method::GET,
                "/auth/verify-email?token=guess",
                [192, 0, 2, 1],
                "text/html",
            ))
            .await
            .expect("request completes");
        assert_eq!(allowed.status(), StatusCode::OK);
    }
    let refused = app
        .clone()
        .oneshot(request(
            Method::GET,
            "/auth/verify-email?token=guess",
            [192, 0, 2, 1],
            "text/html",
        ))
        .await
        .expect("request completes");
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);

    for _ in 0..20 {
        let open = app
            .clone()
            .oneshot(request(Method::GET, "/plans", [192, 0, 2, 1], "text/html"))
            .await
            .expect("request completes");
        assert_eq!(open.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn the_forwarding_header_is_used_only_when_configured() {
    let mut trusting = tight();
    trusting.address_source = AddressSource::ForwardedFor;
    let app = gated_app(trusting);
    let register = <web::auth::Register as ServerFn>::PATH;
    // Every request arrives from the same proxy peer; the header tells the
    // clients apart, so the third client is not refused for the first two.
    for client in ["198.51.100.1", "198.51.100.2", "198.51.100.3"] {
        let mut request = request(Method::POST, register, [10, 0, 0, 2], "text/html");
        request
            .headers_mut()
            .insert("x-forwarded-for", client.parse().unwrap());
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("request completes");
        assert_eq!(response.status(), StatusCode::OK, "{client}");
    }

    let app = gated_app(tight());
    for (index, client) in ["198.51.100.1", "198.51.100.2", "198.51.100.3"]
        .into_iter()
        .enumerate()
    {
        let mut request = request(Method::POST, register, [10, 0, 0, 2], "text/html");
        request
            .headers_mut()
            .insert("x-forwarded-for", client.parse().unwrap());
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("request completes");
        let expected = if index < 2 {
            StatusCode::OK
        } else {
            StatusCode::TOO_MANY_REQUESTS
        };
        assert_eq!(response.status(), expected, "{client} with the peer source");
    }
}
