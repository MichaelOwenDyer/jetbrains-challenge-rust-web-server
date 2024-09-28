//! The application model layer.
//! Contains domain types and connects to the data layer.

use serde::{Deserialize, Serialize};

/// A blog post stored on the server.
#[derive(Debug, Clone, Serialize)]
pub struct BlogPost {
    text: String,
    image_url: Option<String>,
    username: String,
    avatar_url: Option<String>,
    date: time::Date,
}

/// A blog post creation request.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBlogPost {
    text: String,
    image_url: Option<String>,
    username: String,
    avatar_url: Option<String>,
}

impl BlogPost {
    /// Create a blog post from a blog post creation request.
    pub fn create(create: CreateBlogPost) -> Self {
        Self {
            text: create.text,
            image_url: create.image_url,
            username: create.username,
            avatar_url: create.avatar_url,
            date: time::OffsetDateTime::now_utc().date()
        }
    }
}

/// The application service for interacting with the data layer.
#[derive(Debug, Clone)]
pub struct BlogPostController(Vec<BlogPost>);

impl BlogPostController {
    /// Try to create a new BlogPostController.
    pub fn try_new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self(Default::default()))
    }
    /// Save a blog post to the database.
    /// Returns a &'self BlogPost.
    pub async fn save(&mut self, blog_post: BlogPost) -> &BlogPost {
        self.0.push(blog_post);
        &self.0[self.0.len() - 1]
    }
    /// Returns an iterator over all blog posts.
    pub async fn iter(&self) -> impl Iterator<Item = &BlogPost> {
        self.0.iter()
    }
}