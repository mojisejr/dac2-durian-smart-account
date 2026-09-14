//! The network gate: how many times one client address may knock on the
//! account doors in a window before the server asks it to wait.
//!
//! Two budgets. The account budget covers the routes that send mail —
//! registration, verification resend, and the password-reset request —
//! because each one costs a message from a daily allowance and gives a
//! stranger a way to spend it. The login budget covers the routes that take
//! a credential or a token — login, verification, and the reset itself —
//! where the cost of a miss is a guess. Everything else is unlimited.
//!
//! The limiter is one process's memory: a map from budget and address to the
//! moments of that address's recent attempts. That is the right size for a
//! single instance with one worker thread, which is what this application
//! runs; a second instance would count separately, and that is recorded
//! rather than solved here.

use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use leptos::server_fn::ServerFn;

use crate::auth;

/// So many attempts within so long. Written in the environment as
/// `attempts/seconds`, for example `10/300`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limit {
    pub attempts: u32,
    pub window: Duration,
}

impl Limit {
    pub fn parse(name: &str, text: &str) -> Result<Self, String> {
        let error = || format!("{name} must be written as attempts/seconds, for example 10/300");
        let (attempts, seconds) = text.trim().split_once('/').ok_or_else(error)?;
        let attempts: u32 = attempts.trim().parse().map_err(|_| error())?;
        let seconds: u64 = seconds.trim().parse().map_err(|_| error())?;
        if attempts == 0 || seconds == 0 {
            return Err(error());
        }
        Ok(Self {
            attempts,
            window: Duration::from_secs(seconds),
        })
    }
}

/// Which door a request knocks on.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Budget {
    /// Routes that send a message: registration, resend, reset request.
    Account,
    /// Routes that take a credential or a token: login, verify, reset.
    Login,
}

/// Where the client's address is read from. The peer address is the truth
/// when the process faces the network itself; behind a platform proxy every
/// peer is the proxy, and the platform's forwarding header carries the
/// client. Trusting that header is a deployment decision, so it is off until
/// the environment says otherwise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressSource {
    Peer,
    ForwardedFor,
}

impl AddressSource {
    pub fn parse(name: &str, value: Option<String>) -> Result<Self, String> {
        match value.as_deref().map(str::trim) {
            None | Some("") | Some("peer") => Ok(Self::Peer),
            Some("forwarded-for") => Ok(Self::ForwardedFor),
            Some(_) => Err(format!("{name} must be peer or forwarded-for")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateConfig {
    pub account: Limit,
    pub login: Limit,
    pub address_source: AddressSource,
}

impl GateConfig {
    pub const ACCOUNT_DEFAULT: &'static str = "5/3600";
    pub const LOGIN_DEFAULT: &'static str = "10/300";

    /// Reads `RATE_LIMIT_ACCOUNT`, `RATE_LIMIT_LOGIN`, and
    /// `CLIENT_ADDRESS_SOURCE`, each optional, each refused by name when it
    /// does not parse.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let read = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());
        Ok(Self {
            account: Limit::parse(
                "RATE_LIMIT_ACCOUNT",
                &read("RATE_LIMIT_ACCOUNT").unwrap_or_else(|| Self::ACCOUNT_DEFAULT.into()),
            )?,
            login: Limit::parse(
                "RATE_LIMIT_LOGIN",
                &read("RATE_LIMIT_LOGIN").unwrap_or_else(|| Self::LOGIN_DEFAULT.into()),
            )?,
            address_source: AddressSource::parse(
                "CLIENT_ADDRESS_SOURCE",
                read("CLIENT_ADDRESS_SOURCE"),
            )?,
        })
    }

    fn limit(&self, budget: Budget) -> Limit {
        match budget {
            Budget::Account => self.account,
            Budget::Login => self.login,
        }
    }
}

/// The budget a path draws on, if any. Server-function paths come from the
/// functions themselves so a renamed function cannot slip out of the gate.
pub fn budget_for(path: &str) -> Option<Budget> {
    if path == <auth::Register as ServerFn>::PATH
        || path == <auth::ResendVerification as ServerFn>::PATH
        || path == <auth::RequestPasswordReset as ServerFn>::PATH
    {
        Some(Budget::Account)
    } else if path == <auth::Login as ServerFn>::PATH
        || path == <auth::VerifyEmail as ServerFn>::PATH
        || path == <auth::ResetPassword as ServerFn>::PATH
        || path == "/auth/verify-email"
    {
        Some(Budget::Login)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    Allowed,
    /// Wait this long before the next attempt can succeed.
    Refused(Duration),
}

/// The count itself, separated from the middleware so its rules can be
/// exercised with a clock the test controls.
#[derive(Debug)]
pub struct Limiter {
    config: GateConfig,
    attempts: Mutex<HashMap<(Budget, IpAddr), VecDeque<Instant>>>,
}

impl Limiter {
    pub fn new(config: GateConfig) -> Self {
        Self {
            config,
            attempts: Mutex::new(HashMap::new()),
        }
    }

    pub fn config(&self) -> &GateConfig {
        &self.config
    }

    /// Records one attempt at `now` and says whether it may proceed. An
    /// attempt that is refused is not recorded, so a client that keeps
    /// knocking does not push its own release further away.
    pub fn check(&self, budget: Budget, address: IpAddr, now: Instant) -> Decision {
        let limit = self.config.limit(budget);
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // Forgotten addresses are swept when the map has grown past what any
        // honest set of clients produces, so memory is bounded by traffic
        // within one window rather than by traffic since the process started.
        if attempts.len() > 4096 {
            attempts.retain(|(budget, _), moments| {
                let window = self.config.limit(*budget).window;
                moments.retain(|moment| now.duration_since(*moment) < window);
                !moments.is_empty()
            });
        }
        let moments = attempts.entry((budget, address)).or_default();
        while let Some(oldest) = moments.front() {
            if now.duration_since(*oldest) >= limit.window {
                moments.pop_front();
            } else {
                break;
            }
        }
        if moments.len() >= limit.attempts as usize {
            let oldest = moments.front().copied().unwrap_or(now);
            let wait = limit.window.saturating_sub(now.duration_since(oldest));
            return Decision::Refused(wait.max(Duration::from_secs(1)));
        }
        moments.push_back(now);
        Decision::Allowed
    }
}

/// The address a request is counted under, by the configured source. A
/// request that carries no usable address is counted under the unspecified
/// address, so a misconfigured proxy shares one budget instead of none.
pub fn client_address(request: &Request, source: AddressSource) -> IpAddr {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    match source {
        AddressSource::Peer => peer,
        AddressSource::ForwardedFor => request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .and_then(|first| first.trim().parse().ok())
            .or(peer),
    }
    .unwrap_or(IpAddr::from([0, 0, 0, 0]))
}

/// The middleware. Requests to an unguarded path pass untouched.
pub async fn limit_requests(
    State(limiter): State<Arc<Limiter>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(budget) = budget_for(request.uri().path()) else {
        return next.run(request).await;
    };
    let address = client_address(&request, limiter.config().address_source);
    match limiter.check(budget, address, Instant::now()) {
        Decision::Allowed => next.run(request).await,
        Decision::Refused(wait) => refusal(&request, wait),
    }
}

/// The wait, said the way a person would say it.
pub fn wait_text(wait: Duration) -> String {
    let seconds = wait.as_secs().max(1);
    if seconds < 90 {
        format!("{seconds} วินาที")
    } else {
        let minutes = seconds.div_ceil(60);
        format!("{minutes} นาที")
    }
}

pub fn refusal_message(wait: Duration) -> String {
    format!(
        "ลองหลายครั้งติดกันแล้ว กรุณารอ {} แล้วลองใหม่อีกครั้ง",
        wait_text(wait)
    )
}

// A browser that submitted a form without scripts is shown a page it can
// read; the hydrated form, which asks for anything but HTML, gets the same
// sentence in the server-function error encoding so the form displays it
// where it displays every other message.
fn refusal(request: &Request, wait: Duration) -> Response {
    let message = refusal_message(wait);
    let wants_html = request
        .headers()
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"));
    let retry_after = HeaderValue::from(wait.as_secs().max(1));
    if wants_html {
        let body = format!(
            "<!DOCTYPE html><html lang=\"th\"><head><meta charset=\"utf-8\"/>\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\
             <link rel=\"stylesheet\" href=\"/pkg/dac2.css\"/><title>รอสักครู่</title></head>\
             <body><main><section class=\"card recovery-state\"><h1>รอสักครู่</h1>\
             <p>{message}</p><p class=\"muted\">ระบบจำกัดจำนวนครั้ง (rate limit) เพื่อกันการสุ่มรหัสผ่านและการส่งอีเมลซ้ำ</p>\
             <a class=\"button secondary\" href=\"/\">กลับหน้าแรก</a></section></main></body></html>"
        );
        let mut response = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, retry_after);
        response
    } else {
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Body::from(format!("ServerError|{message}")),
        )
            .into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, retry_after);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(last: u8) -> IpAddr {
        IpAddr::from([203, 0, 113, last])
    }

    fn config(account: &str, login: &str) -> GateConfig {
        GateConfig {
            account: Limit::parse("RATE_LIMIT_ACCOUNT", account).unwrap(),
            login: Limit::parse("RATE_LIMIT_LOGIN", login).unwrap(),
            address_source: AddressSource::Peer,
        }
    }

    #[test]
    fn a_limit_is_attempts_over_seconds_and_nothing_else() {
        assert_eq!(
            Limit::parse("RATE_LIMIT_LOGIN", " 10 / 300 "),
            Ok(Limit {
                attempts: 10,
                window: Duration::from_secs(300)
            })
        );
        for bad in ["10", "10/", "/300", "0/300", "10/0", "ten/300", "10/300/1"] {
            assert!(
                Limit::parse("RATE_LIMIT_LOGIN", bad)
                    .unwrap_err()
                    .starts_with("RATE_LIMIT_LOGIN must be written as attempts/seconds"),
                "{bad}"
            );
        }
    }

    #[test]
    fn defaults_apply_when_nothing_is_set_and_a_bad_value_is_refused_by_name() {
        let config = GateConfig::from_lookup(|_| None).unwrap();
        assert_eq!(config.account, Limit::parse("", "5/3600").unwrap());
        assert_eq!(config.login, Limit::parse("", "10/300").unwrap());
        assert_eq!(config.address_source, AddressSource::Peer);

        let error = GateConfig::from_lookup(|name| {
            (name == "RATE_LIMIT_ACCOUNT").then(|| "many".to_string())
        })
        .unwrap_err();
        assert!(error.starts_with("RATE_LIMIT_ACCOUNT must be written"));

        let error = GateConfig::from_lookup(|name| {
            (name == "CLIENT_ADDRESS_SOURCE").then(|| "header".to_string())
        })
        .unwrap_err();
        assert_eq!(error, "CLIENT_ADDRESS_SOURCE must be peer or forwarded-for");
    }

    #[test]
    fn the_budget_engages_after_the_configured_count_and_releases_after_the_window() {
        let limiter = Limiter::new(config("2/60", "3/10"));
        let start = Instant::now();
        let client = address(1);

        for _ in 0..3 {
            assert_eq!(
                limiter.check(Budget::Login, client, start),
                Decision::Allowed
            );
        }
        assert_eq!(
            limiter.check(Budget::Login, client, start + Duration::from_secs(4)),
            Decision::Refused(Duration::from_secs(6))
        );
        // Knocking while refused does not extend the wait.
        assert_eq!(
            limiter.check(Budget::Login, client, start + Duration::from_secs(5)),
            Decision::Refused(Duration::from_secs(5))
        );
        assert_eq!(
            limiter.check(Budget::Login, client, start + Duration::from_secs(10)),
            Decision::Allowed
        );
    }

    #[test]
    fn budgets_and_addresses_are_counted_apart() {
        let limiter = Limiter::new(config("1/60", "1/60"));
        let now = Instant::now();
        assert_eq!(
            limiter.check(Budget::Account, address(1), now),
            Decision::Allowed
        );
        assert!(matches!(
            limiter.check(Budget::Account, address(1), now),
            Decision::Refused(_)
        ));
        assert_eq!(
            limiter.check(Budget::Login, address(1), now),
            Decision::Allowed
        );
        assert_eq!(
            limiter.check(Budget::Account, address(2), now),
            Decision::Allowed
        );
    }

    #[test]
    fn every_account_door_has_a_budget_and_the_rest_of_the_site_has_none() {
        assert_eq!(
            budget_for(<auth::Register as ServerFn>::PATH),
            Some(Budget::Account)
        );
        assert_eq!(
            budget_for(<auth::ResendVerification as ServerFn>::PATH),
            Some(Budget::Account)
        );
        assert_eq!(
            budget_for(<auth::RequestPasswordReset as ServerFn>::PATH),
            Some(Budget::Account)
        );
        assert_eq!(
            budget_for(<auth::Login as ServerFn>::PATH),
            Some(Budget::Login)
        );
        assert_eq!(
            budget_for(<auth::VerifyEmail as ServerFn>::PATH),
            Some(Budget::Login)
        );
        assert_eq!(
            budget_for(<auth::ResetPassword as ServerFn>::PATH),
            Some(Budget::Login)
        );
        assert_eq!(budget_for("/auth/verify-email"), Some(Budget::Login));
        for open in [
            "/",
            "/login",
            "/register",
            "/plans",
            "/plans/3/dashboard",
            "/pkg/dac2.wasm",
        ] {
            assert_eq!(budget_for(open), None, "{open}");
        }
    }

    #[test]
    fn the_wait_is_said_in_seconds_or_whole_minutes() {
        assert_eq!(wait_text(Duration::from_secs(0)), "1 วินาที");
        assert_eq!(wait_text(Duration::from_secs(45)), "45 วินาที");
        assert_eq!(wait_text(Duration::from_secs(90)), "2 นาที");
        assert_eq!(wait_text(Duration::from_secs(3600)), "60 นาที");
    }

    #[test]
    fn the_address_comes_from_the_peer_unless_the_forwarding_header_is_trusted() {
        let mut request = Request::builder()
            .uri("/")
            .header("x-forwarded-for", "198.51.100.7, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 40000))));
        assert_eq!(
            client_address(&request, AddressSource::Peer),
            IpAddr::from([10, 0, 0, 1])
        );
        assert_eq!(
            client_address(&request, AddressSource::ForwardedFor),
            IpAddr::from([198, 51, 100, 7])
        );
        let bare = Request::builder().uri("/").body(Body::empty()).unwrap();
        assert_eq!(
            client_address(&bare, AddressSource::ForwardedFor),
            IpAddr::from([0, 0, 0, 0])
        );
    }
}
