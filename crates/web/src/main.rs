#![recursion_limit = "256"]

// One worker thread on purpose. With the default multi-thread runtime, a
// server-rendered page occasionally sent its shell with the Suspense fallback
// and never sent the resolved chunk, so the browser waited on an open
// connection until its timeout. The stall reproduces without a browser
// (scripts/check-stall.sh: about one page in a hundred while the server also
// streams the wasm bundle) and stops entirely with a single worker, because
// the SSR Suspense machinery in leptos 0.8.20 / reactive_graph 0.2.14 races
// between the thread rendering the page and the thread resolving its
// resource. This is a mitigation, not the fix: the fix belongs upstream, and
// this setting should be revisited when leptos is next upgraded.
//
// A wide worker stack, also on purpose. The release binary renders the
// dashboard and analysis pages through futures deep enough to overflow the
// 2 MiB default worker stack and abort the process; the debug build the
// README's `cargo leptos serve` produces does not, which is why the first
// container run was the first time it showed. Measured on 2026-09-14 against
// the full route matrix: 4 and 8 MiB still overflow, 16 MiB passes; 32 MiB
// leaves headroom, and a stack is reserved address space, not resident
// memory, so it costs nothing on a small host.
#[cfg(feature = "ssr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_stack_size(32 * 1024 * 1024)
        .enable_all()
        .build()?
        .block_on(serve())
}

#[cfg(feature = "ssr")]
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    use axum::{Router, middleware, routing::get};
    use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_sessions_sqlx_store::PostgresStore;
    use web::app::{App, shell};

    // Everything the environment must supply is read and refused by name
    // before the first connection, so a misconfigured host fails at start
    // with the variable's name and never with its value.
    let env = |name: &str| std::env::var(name).ok();
    let database_url = web::settings::required("DATABASE_URL", env("DATABASE_URL"))?;
    let session_key = web::settings::required("SESSION_KEY", env("SESSION_KEY"))?;
    let cookie_secure = web::settings::flag("COOKIE_SECURE", env("COOKIE_SECURE"))?;
    let mail = web::mail::MailConfig::try_from_env()?;
    web::settings::https_requires_secure_cookie(&mail.base_url, cookie_secure)?;
    let gate = web::gate::GateConfig::from_lookup(env)?;
    let pilot_notice = web::settings::flag("PILOT_NOTICE", env("PILOT_NOTICE"))?;
    web::settings::set_pilot_notice(pilot_notice);

    let pool = store::connect(&database_url).await?;
    let row_count = store::migrate_and_probe(&pool).await?;
    println!("database stack probe complete: {row_count} rows");

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let session_key = tower_sessions::cookie::Key::try_from(session_key.as_bytes())
        .map_err(|error| format!("SESSION_KEY must contain at least 64 bytes: {error}"))?;
    // Secure stays off for the localhost workflow, where there is no TLS to
    // carry the cookie; a deployment behind HTTPS sets COOKIE_SECURE=true and
    // the check above refuses an https base URL without it.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_secure(cookie_secure)
        .with_signed(session_key);
    // Where mail goes and how, never a login or a key.
    println!("{}", mail.describe());
    println!(
        "gate: account {}/{}s, login {}/{}s, client address from {}",
        gate.account.attempts,
        gate.account.window.as_secs(),
        gate.login.attempts,
        gate.login.window.as_secs(),
        match gate.address_source {
            web::gate::AddressSource::Peer => "peer",
            web::gate::AddressSource::ForwardedFor => "x-forwarded-for",
        }
    );
    if pilot_notice {
        println!("pilot notice shown");
    }
    let limiter = std::sync::Arc::new(web::gate::Limiter::new(gate));

    let auth_backend = store::AuthBackend::new(pool);
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    // The workspace Cargo.toml carries the development configuration. A
    // container has no workspace, so there the LEPTOS_* variables are the
    // whole configuration; either way an environment variable overrides the
    // file, and PORT, which hosting platforms set, overrides the port alone.
    let manifest = std::path::Path::new("crates/web/Cargo.toml");
    let configuration = get_configuration(manifest.exists().then_some("crates/web/Cargo.toml"))?;
    let mut options = configuration.leptos_options;
    if let Some(port) = web::settings::port(std::env::var("PORT").ok())? {
        options.site_addr = std::net::SocketAddr::new(options.site_addr.ip(), port);
    }
    let address = options.site_addr;
    let routes = generate_route_list(App);
    let app = Router::new()
        .route("/auth/verify-email", get(web::auth::verify_email_link))
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .route_layer(middleware::from_fn(web::auth::require_login))
        .layer(auth_layer)
        // The gate sits outside the session so a refused request never
        // touches the database.
        .layer(middleware::from_fn_with_state(
            limiter,
            web::gate::limit_requests,
        ))
        .with_state(options);

    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("listening on http://{address}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
