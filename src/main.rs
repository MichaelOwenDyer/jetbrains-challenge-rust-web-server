use client::Webapp;
use tracing::info;

mod client;
mod model;
#[cfg(feature = "server")]
mod server;

/// Run the webapp.
/// This function will start the client-side webapp.
#[cfg(all(not(feature = "server"), feature = "web"))]
fn main() {
    dioxus_logger::init(tracing::Level::DEBUG).ok();
    info!("Starting webapp");

    dioxus::launch(Webapp);
}

/// Run the server.
/// This function will connect to the database and start the server.
/// The DATABASE_URL and HOST_ADDR environment variables must be set.
///
/// # Panics
/// This function panics for the following reasons, all of which are considered fatal errors:
/// - If the DATABASE_URL environment variable is not set.
/// - If the HOST_ADDR environment variable is not set.
/// - If the server fails to connect to the database with the specified URL.
/// - If the server fails to open a TCP listener on the specified host address.
/// - If the server fails to start.
#[cfg(all(feature = "server", not(feature = "web")))]
#[tokio::main]
async fn main() {
    use axum::{Extension, Router};
    use dioxus::prelude::*;
    use server::{Database, ServerState};

    // If the logger fails to initialize, we'll just continue without logging.
    dioxus_logger::init(tracing::Level::DEBUG).ok();
    info!("Starting server");

    #[inline]
    fn env(name: &'static str) -> String {
        std::env::var(name)
            .unwrap_or_else(|err| panic!("Failed to read environment variable `{}`: {}", name, err))
    }

    // Load environment variables
    dotenvy::dotenv().ok();
    let database_url = env("DATABASE_URL");
    let host_addr = env("HOST_ADDR");

    // Connect to the database with the specified URL
    let database = Database::try_connect(&database_url)
        .await
        .inspect(|_| info!("Connected to database at {}", database_url))
        .unwrap_or_else(|err| panic!("Failed to connect to database at '{}': {}", database_url, err));

    // Open a TCP listener on the specified host address
    let listener = tokio::net::TcpListener::bind(&host_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to address '{}': {}", host_addr, err));
    info!("Listening on {}", host_addr);

    // Create the router service using the Dioxus application router
    let router_service = Router::new()
        .serve_dioxus_application(
            ServeConfig::builder().build(), 
            || VirtualDom::new(Webapp)
        )
        .await
        // This allows us to extract the database from the request extensions
        .layer(Extension(ServerState { database }))
        .into_make_service();

    // Start the server
    axum::serve(listener, router_service)
        .await
        .unwrap_or_else(|err| panic!("Failed to start server: {}", err));
}
