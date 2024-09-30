use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, From};
use serde::Serialize;
use tracing::{error, trace};

#[derive(Debug, From, Display)]
pub enum AppError {
    DatabaseError(crate::persistence::DatabaseError),
    ImageError(std::io::Error),
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::DatabaseError(err) => {
                error!(%err, "Database error");
                // Client doesn't need to know about what went wrong in the database
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong on our end. Sorry about that!".into()
                )
            }
            AppError::ImageError(err) => {
                // This happens when the client does something wrong
                trace!(%err, "Image error");
                (
                    StatusCode::BAD_REQUEST,
                    format!("{}", err)
                )
            }
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}