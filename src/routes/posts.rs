use crate::error::AppError;
use crate::persistence::model::{BlogPost, BlogPostCreateRequest};
use crate::persistence::sql::Database;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;
use tracing::debug;

pub fn create_router(database: Database) -> Router {
    Router::new()
        .route("/posts", get(get_posts).post(create_post))
        .with_state(Arc::new(database))
}

/// GET /home: Return all blog posts.
async fn get_posts(
    State(controller): State<Arc<Database>>,
) -> Result<(StatusCode, Json<Vec<BlogPost>>), AppError> {
    debug!("--> get_posts");
    let posts = controller.fetch_all()?;
    debug!("<-- get_posts {:?}", posts);
    Ok((StatusCode::OK, Json(posts)))
}

/// POST /home: Create blog post, return 201 Created with created blog post
async fn create_post(
    State(controller): State<Arc<Database>>,
    Json(create_post): Json<BlogPostCreateRequest>,
) -> Result<(StatusCode, Json<BlogPost>), AppError> {
    debug!("--> create_post {:?}", create_post);
    let post = controller.save(create_post)?;
    debug!("<-- create_post {:?}", post);
    Ok((StatusCode::CREATED, Json(post)))
}
