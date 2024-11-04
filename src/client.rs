//! Frontend application code.

use crate::api::*;
use crate::model::{BlogPost, CreateBlogPostParams};
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
    rsx! {
        div { class: "container",
            h1 { class: "header",
                "Welcome to the blog"
            }
            BlogPostForm {}
            BlogPostFeed {}
        }
    }
}

#[component]
fn BlogPostForm() -> Element {
    let mut text_input = use_signal(String::new);
    let mut username_input = use_signal(String::new);
    let mut image_input = use_signal(|| None);
    let mut avatar_input = use_signal(String::new);
    let mut message = use_signal(|| ("red", Cow::from("")));

    let handle_submit = move |_| async move {
        message.set(("yellow", Cow::from("Posting...")));

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
            message.set(("red", Cow::from(msg)));
            return;
        }

        match create_blog_post(params).await {
            Ok(post) => {
                info!("Created post: {:?}", post);
                message.set(("green", Cow::from("Post created!")));
                text_input.set(String::new());
                username_input.set(String::new());
                image_input.set(None);
                avatar_input.set(String::new());
            }
            Err(err) => {
                error!("Failed to create post: {:?}", err);
                message.set(("red", Cow::from(err.to_string())));
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

            // Image File Upload
            div {
                input {
                    r#type: "file",
                    accept: "image/png",
                    onchange: move |evt| {
                        async move {
                            if let Some(file_engine) = evt.files() {
                                let files = file_engine.files();
                                for file_name in &files {
                                    info!("Client picked file: {:?}", file_name);
                                    if let Some(bytes) = file_engine.read_file(file_name).await {
                                        info!("Client read file with size: {}", bytes.len());
                                        image_input.set(Some(bytes));
                                    }
                                }
                            }
                        }
                    },
                }
            }

            // Avatar URL
            div {
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
                label { class: "error",
                    color: "{message().0}",
                    "{message().1}"
                }
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
            h2 { "Post {post.id}" }
            p { "Posted by {post.username}" }
            p { "{post.text}" }
            if let Some(Ok(Some(image))) = &*load_post_image.read_unchecked() {
                img {
                    src: format!("data:image/png;base64,{}", image),
                    alt: "Post image",
                    width: "200",
                }
            }
            if let Some(Ok(Some(avatar))) = &*load_avatar_image.read_unchecked() {
                img {
                    src: format!("data:image/png;base64,{}", avatar),
                    alt: "Avatar",
                    width: "50",
                }
            }
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
