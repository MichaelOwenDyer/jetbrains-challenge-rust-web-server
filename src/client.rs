//! Frontend application code.

use crate::model::{AvatarImageUuid, BlogPost, BlogPostId, CreateBlogPostParams, PostImageUuid};
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use std::borrow::Cow;

/// The routes for the frontend application.
/// / or /home -> HomePage
/// /... -> PageNotFound
#[derive(Debug, Clone, Routable)]
enum Route {
    #[redirect("/", || Route::HomePage)]
    #[route("/home")]
    HomePage,
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

/// The main entry point for the frontend application.
#[allow(non_snake_case)]
pub fn Webapp() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "Page not found" }
        p { "The page you requested doesn't exist." }
        pre { color: "red", "Attempted to navigate to: {route:?}" }
    }
}

#[component]
fn HomePage() -> Element {
    rsx! {
        div {
            h1 { "Welcome to the blog" }
            BlogPostForm {}
            BlogPostFeed {}
        }
    }
}

#[component]
fn BlogPostForm() -> Element {
    let mut input = use_signal(CreateBlogPostParams::default);
    let mut error_msg = use_signal(|| Cow::from(""));

    let submit_post = move |_| {
        let params = input();

        async move {
            match create_blog_post(params).await {
                Ok(post) => {
                    info!("Created post: {:?}", post);
                    error_msg.set(Cow::from(""));
                    input.set(CreateBlogPostParams::default());
                }
                Err(err) => {
                    info!("Failed to create post: {:?}", err);
                    error_msg.set(Cow::from(format!("Failed to create post: {:?}", err)));
                }
            }
        }
    };

    rsx! {
        div {
            form {
                border: "1px solid black",
                h3 { "Username" }
                input {
                    value: "{input.read().username}",
                    oninput: move |e| input.write().username = e.value(),
                }
                h3 { "Post text" }
                textarea {
                    value: "{input.read().text}",
                    oninput: move |e| input.write().text = e.value(),
                }
                h3 { "Image" }
                FileUpload {
                    accept: ".png",
                    onchange: move |bytes: Vec<u8>| {
                        input.write().image = Some(bytes);
                    },
                }
                h3 { "Avatar Image URL" }
                input {
                    r#type: "url",
                    accept: ".png",
                    onchange: move |url| input.write().avatar_url = Some(url.value()),
                }
                button {
                    r#type: "submit",
                    onclick: submit_post,
                    "Submit Post"
                }
            }
            div {
                color: "red",
                "{error_msg}"
            }
        }
    }
}

#[component]
fn FileUpload(
    accept: &'static str,
    #[props(default = false)] multiple: bool,
    onchange: EventHandler<Vec<u8>>,
) -> Element {
    rsx! {
        input {
            r#type: "file",
            accept: accept,
            multiple: multiple,
            onchange: move |evt| {
                async move {
                    if let Some(file_engine) = evt.files() {
                        let files = file_engine.files();
                        for file_name in &files {
                            info!("Client picked file: {:?}", file_name);
                            if let Some(bytes) = file_engine.read_file(file_name).await {
                                info!("Client read file with size: {}", bytes.len());
                                onchange(bytes);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FileUrlInput(accept: &'static str, onchange: EventHandler<String>) -> Element {
    rsx! {}
}

#[component]
fn BlogPostFeed() -> Element {
    let fetch_posts = use_resource(fetch_blog_posts);

    match &*fetch_posts.read_unchecked() {
        Some(Ok(posts)) => rsx! {
            div {
                h1 { "Blog Posts" }
                ul {
                    for (id, post) in posts.iter().map(|post| post.id).zip(posts.iter()) {
                        li { key: "{id}",
                            Post { post: post.clone() }
                        }
                    }
                }
            }
        },
        Some(Err(_)) => rsx! {
            div {
                h2 {
                    color: "red",
                    "Error fetching posts"
                }
            }
        },
        None => rsx! {
            div {
                h2 {
                    color: "gray",
                    "Loading posts..."
                }
            }
        },
    }
}

#[component]
fn Post(post: BlogPost) -> Element {
    rsx! {
        div {
            h2 { "Post {post.id}" }
            p { "Posted by {post.username}" }
            p { "{post.text}" }
            // if let Some(Ok(Some(bytes))) = &*fetch_post_image.read_unchecked() {
            //     img {
            //         src: format!("data:image/png;base64,{}", base64::encode(bytes)),
            //         alt: "Post image",
            //         width: "200",
            //     }
            // }
            // if let Some(Ok(Some(avatar))) = &*fetch_avatar_image.read_unchecked() {
            //     img {
            //         src: format!("data:image/png;base64,{}", base64::encode(avatar)),
            //         alt: "Avatar image",
            //         width: "50",
            //     }
            // }
            button {
                onclick: move |_| async move {
                    if delete_blog_post(post.id).await.is_ok() {
                        info!("Client deleted post with id: {}", post.id);
                    } else {
                        // TODO: Notify the user of the error
                        error!("Client failed to delete post with id: {}", post.id);
                    }
                },
                "Delete"
            }
        }
    }
}

// Server functions.
// These functions represent the API endpoints that the frontend application interacts with.

/// API endpoint to fetch all blog posts.
/// TODO: Implement pagination and streaming.
#[server(FetchBlogPosts)]
async fn fetch_blog_posts() -> Result<Vec<BlogPost>, ServerFnError> {
    use crate::server::Database;
    let database: Database = extract().await?;
    let posts = database.fetch_all().await?;
    Ok(posts)
}

/// API endpoint to create a blog post.
#[server(CreateBlogPost)]
async fn create_blog_post(params: CreateBlogPostParams) -> Result<BlogPost, ServerFnError> {
    use crate::server::{images, Database};
    use crate::model::InsertBlogPost;
    let database: Database = extract().await?;
    // Save images to the file system and get their UUIDs
    let (image_uuid, avatar_uuid) = images::process_images(params.image, params.avatar_url).await?;
    // Insert the blog post into the database
    let to_persist = InsertBlogPost::new(params.text, params.username, image_uuid, avatar_uuid);
    let post = database.save(to_persist).await?;
    Ok(post)
}

/// API endpoint to delete a blog post.
#[server(DeleteBlogPost)]
async fn delete_blog_post(post_id: BlogPostId) -> Result<(), ServerFnError> {
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
#[server(LoadPostImage)]
async fn load_post_image(uuid: PostImageUuid) -> Result<Vec<u8>, ServerFnError> {
    crate::server::images::load(&uuid).await.map_err(Into::into)
}

/// API endpoint to fetch an avatar image.
#[server(LoadAvatarImage)]
async fn load_avatar_image(uuid: AvatarImageUuid) -> Result<Vec<u8>, ServerFnError> {
    crate::server::images::load(&uuid).await.map_err(Into::into)
}
