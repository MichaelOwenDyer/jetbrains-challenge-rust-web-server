//! Server-specific functionality.

use axum::async_trait;
use std::convert::Infallible;

pub mod images;
pub mod persistence;

pub use persistence::database::Database;

/// The state of the server.
/// For now this only holds the database, but it could hold more in the future.
#[derive(Debug, Clone)]
pub struct ServerState {
    pub database: Database,
}

/// Enable the database to be extracted from the request extensions.
#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for Database {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Infallible> {
        let server_state: ServerState = parts
            .extensions
            .get()
            .cloned()
            // Safety: We know that the server state is present because we put it there.
            // See Router creation in main.rs
            .expect("Server state should be present in request extensions");
        Ok(server_state.database)
    }
}
