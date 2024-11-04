//! Data models for the blog post application.

use serde::{Deserialize, Serialize};

/// Blog post creation data sent by the client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateBlogPostParams {
    /// The text of the blog post.
    pub text: String,
    /// The username of the author of the blog post.
    pub username: String,
    /// The bytes of the file attached to the blog post, if any.
    /// These bytes have not yet been validated to ensure they are an image.
    pub image: Option<Vec<u8>>,
    /// The URL of the author's avatar, if any.
    /// This URL has not yet been validated to ensure it is an image.
    pub avatar_url: Option<String>,
}

impl CreateBlogPostParams {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.text.is_empty() {
            return Err("The blog post text cannot be empty");
        }
        if self.username.is_empty() {
            return Err("The username cannot be empty");
        }
        Ok(())
    }
}

/// The file system path of a blog post image.
/// This is a newtype around a `String`, which is the UUID of the image.
/// The UUID is persisted to the database, and is used to load the image from the file system later.
/// We cannot use the `Uuid` type directly because SQLite does not support it with Diesel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server",
    derive(diesel::FromSqlRow, diesel::AsExpression),
    diesel(sql_type = diesel::sql_types::Text)
)]
pub struct PostImagePath(pub String);

/// The file system path of an avatar image.
/// This is a newtype around a `String`, which is the UUID of the image.
/// The UUID is persisted to the database, and is used to load the image from the file system later.
/// We cannot use the `Uuid` type directly because SQLite does not support it with Diesel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server",
    derive(diesel::FromSqlRow, diesel::AsExpression),
    diesel(sql_type = diesel::sql_types::Text)
)]
pub struct AvatarImagePath(pub String);

#[cfg(feature = "server")]
pub use server::*;

/// Server-specific models and functionality.
#[cfg(feature = "server")]
mod server {
    use super::{AvatarImagePath, PostImagePath};
    use diesel::{backend::Backend, deserialize, serialize, sql_types::Text};

    /// Implement the necessary Diesel traits for an image UUID newtype.
    macro_rules! impl_image {
        ($name:ident) => {
            impl<B: Backend> serialize::ToSql<Text, B> for $name
            where
                String: serialize::ToSql<Text, B>,
            {
                fn to_sql<'b>(
                    &'b self,
                    out: &mut serialize::Output<'b, '_, B>,
                ) -> serialize::Result {
                    self.0.to_sql(out)
                }
            }

            impl<B: Backend> deserialize::FromSql<Text, B> for $name
            where
                String: deserialize::FromSql<Text, B>,
            {
                fn from_sql(bytes: B::RawValue<'_>) -> deserialize::Result<Self> {
                    let uuid = String::from_sql(bytes)?;
                    Ok($name(uuid))
                }
            }
        };
    }

    impl_image!(PostImagePath);
    impl_image!(AvatarImagePath);

    /// Insertable data for a blog post.
    #[derive(Debug, diesel::Insertable)]
    #[diesel(table_name = crate::server::persistence::schema::blog_post)]
    pub struct InsertBlogPost {
        pub posted_on: time::Date,
        pub text: String,
        pub username: String,
        pub image_uuid: Option<PostImagePath>,
        pub avatar_uuid: Option<AvatarImagePath>,
    }

    impl InsertBlogPost {
        pub fn new(
            text: String,
            username: String,
            image_uuid: Option<PostImagePath>,
            avatar_uuid: Option<AvatarImagePath>,
        ) -> Self {
            Self {
                posted_on: time::OffsetDateTime::now_utc().date(),
                text,
                username,
                image_uuid,
                avatar_uuid,
            }
        }
    }
}

/// The ID of a blog post.
pub type BlogPostId = i32;

/// A blog post that has been saved to the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "server",
    derive(diesel::Queryable, diesel::Selectable),
    diesel(table_name = crate::server::persistence::schema::blog_post),
    diesel(check_for_backend(diesel::sqlite::Sqlite))
)]
pub struct BlogPost {
    pub id: BlogPostId,
    pub posted_on: time::Date,
    pub text: String,
    pub username: String,
    pub image_uuid: Option<PostImagePath>,
    pub avatar_uuid: Option<AvatarImagePath>,
}
