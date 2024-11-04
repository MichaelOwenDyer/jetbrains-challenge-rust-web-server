//! API endpoints for the application client to interact with the server.
//! These functions are compiled differently for the server and client binaries.
//! For the server, they are compiled as they appear here.
//! For the client, they are compiled as API calls to the server.
//! This is the reason for the local imports in this module.

use crate::model::{AvatarImagePath, BlogPost, BlogPostId, CreateBlogPostParams, PostImagePath};
use dioxus::prelude::*;

/// API endpoint to fetch all blog posts.
/// TODO: Implement pagination and streaming.
#[server(FetchBlogPosts)]
pub async fn fetch_blog_posts() -> Result<Vec<BlogPost>, ServerFnError> {
    use crate::server::Database;
    
    // Fetch all blog posts from the database
    let database: Database = extract().await?;
    let posts = database.fetch_all().await?;
    Ok(posts)
}

/// API endpoint to create a blog post.
#[server(CreateBlogPost)]
pub async fn create_blog_post(params: CreateBlogPostParams) -> Result<BlogPost, ServerFnError> {
    use crate::model::InsertBlogPost;
    use crate::server::{images, Database};
    use tracing::debug;
    
    debug!("Creating blog post");
    let database: Database = extract().await?;
    // Save images to the file system and get their UUIDs
    debug!("Processing images");
    let (image_uuid, avatar_uuid) = images::process_images(params.image, params.avatar_url).await?;
    debug!("Images processed: image: {image_uuid:?}, avatar: {avatar_uuid:?}");
    // Insert the blog post into the database
    let to_persist = InsertBlogPost::new(params.text, params.username, image_uuid, avatar_uuid);
    let post = database.save(to_persist).await?;
    Ok(post)
}

/// API endpoint to delete a blog post.
#[server(DeleteBlogPost)]
pub async fn delete_blog_post(post_id: BlogPostId) -> Result<(), ServerFnError> {
    use crate::server::{images, Database};
    
    let database: Database = extract().await?;
    let deleted = database.delete(post_id).await?;
    // Try to delete the images from the file system
    // It's not a big deal if this fails, so we ignore the result
    let _ = tokio::join!(
        images::delete(deleted.image_uuid.as_ref()),
        images::delete(deleted.avatar_uuid.as_ref())
    );
    Ok(())
}

// Image server functions.
// Server functions cannot be generic, so we need to define a separate function for each image type.

/// API endpoint to fetch a post image.
/// The image is returned as a base64-encoded string.
#[server(LoadPostImage)]
pub async fn load_post_image(uuid: PostImagePath) -> Result<String, ServerFnError> {
    use base64::Engine;
    crate::server::images::load(&uuid)
        .await
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
        .map_err(Into::into)
}

/// API endpoint to fetch an avatar image.
/// The image is returned as a base64-encoded string.
#[server(LoadAvatarImage)]
pub async fn load_avatar_image(uuid: AvatarImagePath) -> Result<String, ServerFnError> {
    use base64::Engine;
    crate::server::images::load(&uuid)
        .await
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
        .map_err(Into::into)
}
