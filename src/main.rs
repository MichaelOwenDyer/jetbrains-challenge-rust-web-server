use crate::model::{BlogPost, BlogPostController, CreateBlogPost};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use tracing_subscriber::filter::LevelFilter;

mod model;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let datastore = BlogPostController::try_new().expect("Failed to create datastore");

    let app = Router::new()
        .route("/home", get(get_posts).post(create_post))
        .with_state(Arc::new(Mutex::new(datastore)));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 8080))
        .await
        .unwrap();
    info!("Listening on port 8080");
    axum::serve(listener, app).await.unwrap();
}

/// GET /home: Return all blog posts.
async fn get_posts(
    State(controller): State<Arc<Mutex<BlogPostController>>>,
) -> (StatusCode, Json<Vec<BlogPost>>) {
    debug!("--> get_posts");
    let posts = controller.lock().await.iter().await.cloned().collect();
    debug!("<-- get_posts {:?}", posts);
    (StatusCode::OK, Json(posts))
}

/// POST /home: Create blog post, return 201 Created with created blog post
async fn create_post(
    State(controller): State<Arc<Mutex<BlogPostController>>>,
    Json(create_post): Json<CreateBlogPost>,
) -> (StatusCode, Json<BlogPost>) {
    debug!("--> create_post {:?}", create_post);
    let post = BlogPost::create(create_post);
    let post = controller.lock().await.save(post).await.clone();
    debug!("<-- create_post {:?}", post);
    (StatusCode::CREATED, Json(post))
}
