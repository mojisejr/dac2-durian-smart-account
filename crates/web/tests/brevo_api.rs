#![cfg(feature = "ssr")]

//! The Brevo HTTP delivery against a local stand-in for the API: the request
//! carries the key in the `api-key` header and the JSON shape Brevo documents,
//! a 201 is success, and a refusal is reported by status alone, without the
//! body that would echo the recipient.

use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use web::mail::{Delivery, MailConfig, Mailer};

#[derive(Clone, Default)]
struct Seen {
    api_key: Option<String>,
    body: Option<serde_json::Value>,
}

async fn stand_in(
    State(state): State<(Arc<Mutex<Seen>>, StatusCode)>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let (seen, status) = state;
    let mut seen = seen.lock().unwrap();
    seen.api_key = headers
        .get("api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    seen.body = Some(body.clone());
    (
        status,
        Json(serde_json::json!({ "echo": body, "message": "recipient echoed here on purpose" })),
    )
}

async fn serve(status: StatusCode) -> (String, Arc<Mutex<Seen>>) {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let app = Router::new()
        .route("/v3/smtp/email", post(stand_in))
        .with_state((seen.clone(), status));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a free port");
    let address = listener.local_addr().expect("bound address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("stand-in serves");
    });
    (format!("http://{address}/v3/smtp/email"), seen)
}

fn mailer(url: &str) -> Mailer {
    Mailer::with_brevo_url(
        MailConfig {
            delivery: Delivery::BrevoApi {
                api_key: "xkeysib-stand-in".into(),
            },
            from: "บัญชีตาไก๊ <no-reply@dac2.local>".into(),
            base_url: "https://dac2-pilot.example".into(),
        },
        url,
    )
    .expect("the HTTP client builds")
}

#[tokio::test]
async fn a_verification_mail_reaches_the_api_with_key_sender_recipient_and_link() {
    let (url, seen) = serve(StatusCode::CREATED).await;
    mailer(&url)
        .send_verification("owner@example.test", "tok3n")
        .await
        .expect("201 is success");

    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.api_key.as_deref(), Some("xkeysib-stand-in"));
    let body = seen.body.expect("a JSON body was posted");
    assert_eq!(body["sender"]["email"], "no-reply@dac2.local");
    assert_eq!(body["sender"]["name"], "บัญชีตาไก๊");
    assert_eq!(body["to"][0]["email"], "owner@example.test");
    assert_eq!(body["subject"], "ยืนยันอีเมล บัญชีตาไก๊");
    let text = body["textContent"].as_str().expect("text content");
    assert!(
        text.contains("https://dac2-pilot.example/auth/verify-email?token=tok3n"),
        "{text}"
    );
}

#[tokio::test]
async fn a_refusal_is_reported_by_status_without_the_echoed_body() {
    let (url, _seen) = serve(StatusCode::UNAUTHORIZED).await;
    let error = mailer(&url)
        .send_password_reset("owner@example.test", "tok3n")
        .await
        .expect_err("401 is a failure");
    let message = error.to_string();
    assert_eq!(message, "Brevo API answered 401 Unauthorized");
    assert!(!message.contains("owner@example.test"));
    assert!(!message.contains("xkeysib"));
}
