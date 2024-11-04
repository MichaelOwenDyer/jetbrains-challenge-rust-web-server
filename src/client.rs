//! Frontend application code.

use crate::api::*;
use crate::model::{BlogPost, CreateBlogPostParams};
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use std::borrow::Cow;
use tracing::debug;

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
        body {
            Router::<Route> {}
        }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "container",
            h1 { "Page not found" }
            p { "The page you requested doesn't exist." }
        }
    }
}

#[component]
fn HomePage() -> Element {
    let mut fetch_blog_posts = use_resource(fetch_blog_posts);
    rsx! {
        div { class: "container",
            h1 { class: "header",
                "Welcome to the blog"
            }
            BlogPostForm {
                oncreate: move |_| fetch_blog_posts.restart(),
            }
            BlogPostFeed {
                posts: fetch_blog_posts.read_unchecked().clone(),
            }
        }
    }
}

#[component]
fn BlogPostForm(
    oncreate: EventHandler<BlogPost>,
) -> Element {
    let mut text_input = use_signal(String::new);
    let mut username_input = use_signal(String::new);
    let mut image_input = use_signal(|| None);
    let mut avatar_input = use_signal(String::new);
    let mut message = use_signal(|| ("red", None));

    let handle_submit = move |_| async move {
        message.set(("yellow", Some(Cow::from("Posting..."))));

        let params = CreateBlogPostParams {
            text: text_input().clone(),
            username: username_input().clone(),
            image: image_input().clone(),
            avatar_url: if avatar_input().is_empty() {
                None
            } else {
                Some(avatar_input().clone())
            },
        };

        if let Err(msg) = params.validate() {
            message.set(("red", Some(Cow::from(msg))));
            return;
        }

        match create_blog_post(params).await {
            Ok(post) => {
                info!("Created post: {:?}", post);
                message.set(("green", Some(Cow::from("Post created!"))));
                text_input.set(String::new());
                username_input.set(String::new());
                image_input.set(None);
                avatar_input.set(String::new());
                oncreate(post);
            }
            Err(err) => {
                error!("Failed to create post: {:?}", err);
                message.set(("red", Some(Cow::from(err.to_string()))));
            }
        }
    };

    rsx! {
        form { class: "blog-post-form",

            // Username Input
            div {
                label { "Who are you?" }
                label { "What's on your mind?" }
            }

            // Text Area for the Post
            div {
                input {
                    r#type: "text",
                    value: "{username_input}",
                    placeholder: "Enter your username",
                    oninput: move |evt| username_input.set(evt.value()),
                }
                textarea {
                    value: "{text_input}",
                    placeholder: "Write your post here...",
                    oninput: move |evt| text_input.set(evt.value()),
                }
            }

            div {
                // Image File Upload
                input {
                    r#type: "file",
                    accept: "image/png",
                    onchange: move |evt| {
                        async move {
                            if let Some(file_engine) = evt.files() {
                                let files = file_engine.files();
                                for file_name in &files {
                                    debug!("User picked file: {:?}", file_name);
                                    if let Some(bytes) = file_engine.read_file(file_name).await {
                                        debug!("Uploaded {}B", bytes.len());
                                        image_input.set(Some(bytes));
                                    }
                                }
                            }
                        }
                    },
                }

                // Avatar URL
                input {
                    r#type: "url",
                    value: "{avatar_input}",
                    placeholder: "Avatar URL (optional)",
                    oninput: move |evt| avatar_input.set(evt.value()),
                }
                if !avatar_input().is_empty() {
                    div {
                        img {
                            src: "{avatar_input}",
                            alt: "Avatar",
                            width: "50",
                        }
                    }
                }
            }

            // Submit Button
            div {
                button { class: "post-btn",
                    r#type: "submit",
                    prevent_default: "onclick",
                    onclick: handle_submit,
                    "Submit Post"
                }
                if let Some(error_msg) = message().1 {
                    div { class: "error",
                        color: "{message().0}",
                        "{error_msg}"
                    }
                }
            }
        }
    }
}

#[component]
fn BlogPostFeed(
    posts: Option<Result<Vec<BlogPost>, ServerFnError>>,
) -> Element {
    match posts {
        Some(Ok(posts)) => {
            let posts = posts.into_iter().map(|post| {
                let deleted = use_signal(|| false);
                (post, deleted)
            });
            rsx! {
                div {
                    h2 { "Recent Posts" }
                    ul {
                        for (post, deleted) in posts {
                            li { key: "{post.id.clone()}", hidden: deleted,
                                Post { post, deleted }
                            }
                        }
                    }
                }
            }
        },
        Some(Err(_)) => rsx! {
            div {
                h2 { color: "red",
                    "Error fetching posts"
                }
            }
        },
        None => rsx! {
            div {
                h2 { color: "gray",
                    "Loading posts..."
                }
            }
        },
    }
}

#[component]
fn Post(
    post: BlogPost,
    deleted: Signal<bool>,
) -> Element {
    let post_image_uuid = post.image_uuid.clone();
    let load_post_image = use_resource(move || {
        let post_image_uuid = post_image_uuid.clone();
        async move {
            if let Some(image_uuid) = &post_image_uuid {
                load_post_image(image_uuid.clone()).await.map(Some)
            } else {
                Ok(None)
            }
        }
    });
    let avatar_image_uuid = post.avatar_uuid.clone();
    let load_avatar_image = use_resource(move || {
        let avatar_image_uuid = avatar_image_uuid.clone();
        async move {
            if let Some(avatar_uuid) = &avatar_image_uuid {
                load_avatar_image(avatar_uuid.clone()).await.map(Some)
            } else {
                Ok(None)
            }
        }
    });
    rsx! {
        div {
            h3 { "Post {post.id}" }
            p { "Posted by {post.username} on {post.posted_on}" }
            if let Some(Ok(Some(avatar))) = &*load_avatar_image.read_unchecked() {
                img {
                    src: format!("data:image/png;base64,{}", avatar),
                    alt: "Avatar",
                    width: "50",
                }
            }
            p { "{post.text}" }
            if let Some(Ok(Some(image))) = &*load_post_image.read_unchecked() {
                img {
                    src: format!("data:image/png;base64,{}", image),
                    alt: "Post image",
                    width: "200",
                }
            }
            div { class: "blog-post-actions",
                button {
                    onclick: move |_| async move {
                        if delete_blog_post(post.id).await.is_ok() {
                            info!("Deleted post with id: {}", post.id);
                            deleted.set(true);
                        } else {
                            error!("Failed to delete post with id: {}", post.id);
                        }
                    },
                    "Delete"
                }
            }
        }
    }
}
