use crate::persistence::sql::Database;
use axum::Router;
use clap::Parser;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

mod error;
mod persistence;
mod routes;

#[derive(Debug, Parser)]
struct AppArgs {
    /// The port to host the HTTP server on.
    #[arg(long, default_value_t = 8080)]
    port: u16,
    /// The URL of the database. If not set, the DATABASE_URL environment variable is used.
    #[arg(long)]
    database_url: Option<String>,
    /// The logging level, one of TRACE, DEBUG, INFO, WARN, ERROR. Defaults to INFO.
    #[arg(long, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse();

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::from_level(args.log_level))
        .init();

    let database_url = match args.database_url {
        Some(database_url) => database_url,
        None => std::env::var("DATABASE_URL")?,
    };
    let database = Database::try_connect(&database_url).await?;

    let app = Router::new()
        .merge(routes::posts::create_router(database))
        .merge(routes::images::create_router());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    info!("Listening on port {}", args.port);
    axum::serve(listener, app).await?;
    Ok(())
}
