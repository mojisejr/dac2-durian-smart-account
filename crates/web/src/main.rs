#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::Router;
    use axum_login::{AuthManagerLayerBuilder, tower_sessions::SessionManagerLayer};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_sessions_sqlx_store::PostgresStore;
    use web::app::{App, shell};

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = store::connect(&database_url).await?;
    let row_count = store::migrate_and_probe(&pool).await?;
    println!("database stack probe complete: {row_count} rows");

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
    let auth_backend = store::StackAuthBackend::new(pool);
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let configuration = get_configuration(Some("crates/web/Cargo.toml"))?;
    let address = configuration.leptos_options.site_addr;
    let options = configuration.leptos_options;
    let routes = generate_route_list(App);
    let app = Router::new()
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(auth_layer)
        .with_state(options);

    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("listening on http://{address}");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
