use client::Webapp;
use tracing::info;

mod api;
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
/// The DATABASE_URL environment variable must be set.
/// The LOG_LEVEL environment variable is optional and defaults to INFO.
/// The HOST_ADDR environment variable is optional and defaults to "0.0.0.0:8080".
/// The server will listen on the specified host address.
///
/// # Panics
/// This function panics for the following reasons, all of which are considered fatal errors:
/// - If the LOG_LEVEL environment variable is set but fails to parse.
/// - If the DATABASE_URL environment variable is not set.
/// - If the server fails to connect to the database with the specified URL.
/// - If the server fails to open a TCP listener on the specified host address.
/// - If the axum server fails to start.
#[cfg(all(feature = "server", not(feature = "web")))]
#[tokio::main]
async fn main() {
    use std::env::var as env;
    use axum::{Extension, Router};
    use dioxus::prelude::*;
    use server::{Database, ServerState};
    
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Load the log level from the environment variable or use the default
    let log_level = match env("LOG_LEVEL") {
        Ok(level) => level.parse().unwrap_or_else(|err| {
            panic!("Failed to parse log level from environment variable `LOG_LEVEL`: {}", err)
        }),
        Err(_) => tracing::Level::INFO,
    };
    
    // If the logger fails to initialize, we'll just continue without logging.
    dioxus_logger::init(log_level).ok();
    info!("Starting server");

    // Load the database URL from the environment variable
    let database_url = env("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");
    // Connect to the database with the specified URL
    let database = Database::try_connect(&database_url)
        .await
        .inspect(|_| info!("Connected to database at {database_url}"))
        .unwrap_or_else(|err| panic!("Failed to connect to database at '{database_url}': {err}"));

    // Load the host address from the environment variable or use the default
    let host_addr = env("HOST_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    // Open a TCP listener on the specified host address
    let listener = tokio::net::TcpListener::bind(&host_addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to address '{}': {}", host_addr, err));
    info!("Listening on {}", host_addr);

    // Create the router service using the Dioxus application router
    let router_service = Router::new()
        .serve_dioxus_application(ServeConfig::builder().build(), || VirtualDom::new(Webapp))
        .await
        // This allows us to extract the database from the request extensions
        .layer(Extension(ServerState { database }))
        .into_make_service();

    // Start the server
    axum::serve(listener, router_service)
        .await
        .unwrap_or_else(|err| panic!("Failed to start server: {}", err));
}
