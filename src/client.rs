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
    let mut text_input = use_signal(String::new);
    let mut username_input = use_signal(String::new);
    let mut image_input = use_signal(|| None);
    let mut avatar_input = use_signal(String::new);
    let mut error_msg = use_signal(|| Cow::from(""));

    let handle_submit = move |_| {
        async move {
            let text = text_input.read();
            let username = username_input.read();
            if text.is_empty() {
                error_msg.set(Cow::from("Post text cannot be empty"));
                return;
            }
            if username.is_empty() {
                error_msg.set(Cow::from("Username cannot be empty"));
                return;
            }
            let image = image_input.read();
            let avatar_url = avatar_input.read();
            let params = CreateBlogPostParams {
                text: text.clone(),
                username: username.clone(),
                image: image.clone(),
                avatar_url: if avatar_url.is_empty() {
                    None
                } else {
                    Some(avatar_url.clone())
                },
            };

            match create_blog_post(params).await {
                Ok(post) => {
                    info!("Created post: {:?}", post);
                    error_msg.set(Cow::from(""));
                    // text.set(String::new());
                    // username.set(String::new());
                    // image.set(None);
                    // avatar_url.set(String::new());
                }
                Err(err) => {
                    error!("Failed to create post: {:?}", err);
                    error_msg.set(Cow::from(format!("Failed to create post: {:?}", err)));
                }
            }
        }
    };

    rsx! {
        form {
            // prevent_default: "onsubmit",
            // onsubmit: handle_submit,
            class: "blogpost-form",

            // Username Input
            div { class: "input-container",
                label { "Username:" }
                input {
                    r#type: "text",
                    value: "{username_input}",
                    placeholder: "Enter your username",
                    oninput: move |evt| username_input.set(evt.value()),
                }
            }

            // Text Area for the Post
            div { class: "input-container",
                label { "What's on your mind?" }
                div {
                    textarea {
                        value: "{text_input}",
                        placeholder: "Write your post here...",
                        oninput: move |evt| text_input.set(evt.value()),
                    }
                }
            }

            // Image File Upload
            div { class: "input-container",
                label { "Upload Image:" }
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
            div { class: "input-container",
                label { "Avatar URL (optional):" }
                input {
                    r#type: "url",
                    value: "{avatar_input}",
                    placeholder: "Enter your avatar URL",
                    oninput: move |evt| avatar_input.set(evt.value()),
                }
            }

            // Submit Button
            div { class: "input-container",
                button {
                    r#type: "submit",
                    prevent_default: "onclick",
                    onclick: handle_submit,
                    "Submit Post"
                }
            }
        }
        div {
            color: "red",
            "{error_msg}"
        }
    }
}

#[component]
fn BlogPostForm1() -> Element {
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
