//! The application model layer.
//! Contains domain types and connects to the data layer.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::persistence::schema::blog_post;

/// A blog post creation request.
#[derive(Debug, Clone, Deserialize)]
pub struct BlogPostCreateRequest {
    pub text: String,
    pub username: String,
    pub image_url: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = blog_post)]
pub struct InsertBlogPost {
    pub posted_on: time::Date,
    pub text: String,
    pub username: String,
    pub image_uuid: Option<String>,
    pub avatar_uuid: Option<String>,
}

/// A blog post stored in the database.
#[derive(Debug, Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = blog_post)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BlogPost {
    pub id: i32,
    pub posted_on: time::Date,
    pub text: String,
    pub username: String,
    pub image_uuid: Option<String>,
    pub avatar_uuid: Option<String>,
}