use axum::Router;
use axum::routing::get_service;
use tower_http::services::ServeDir;
use crate::error::AppError;

pub fn create_router() -> Router {
    Router::new()
        .route(
            "/images/posts", 
            get_service(ServeDir::new("/images/posts"))
        )
        .route(
            "/images/avatars",
            get_service(ServeDir::new("/images/avatars"))
        )
}

#[expect(unused)]
async fn handle_error(err: std::io::Error) -> AppError {
    AppError::ImageError(err)
}