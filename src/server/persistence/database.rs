//! Database module for interacting with the SQLite database.

use crate::model::{BlogPost, BlogPostId, InsertBlogPost};
use crate::server::persistence::schema::blog_post::dsl::*;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, info};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum DatabaseError {
    #[display("Connection error: {}", _0)]
    Connection(r2d2::Error),
    #[display("Migration error: {}", _0)]
    Migration(Box<dyn std::error::Error + Send + Sync>),
    #[display("SQL error: {}", _0)]
    Sql(diesel::result::Error),
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    /// Attempt to connect to the SQLite database at the provided URL.
    /// Create a connection pool and immediately run embedded Diesel migrations
    /// to ensure the schema is up-to-date.
    /// Return a `Database` instance if successful.
    /// Returns `DatabaseError::Connection` if connecting to the database fails.
    /// Returns `DatabaseError::Migration` if migrating the database fails.
    pub async fn try_connect(url: impl Into<String>) -> Result<Self, DatabaseError> {
        let url = url.into();
        tokio::task::spawn_blocking(move || {
            let pool = r2d2::Pool::builder()
                .max_size(5)
                .build(ConnectionManager::<SqliteConnection>::new(url))?;
            let mut conn = pool.get()?;
            let versions = conn.run_pending_migrations(MIGRATIONS)?;
            if !versions.is_empty() {
                info!("Successfully updated database schema.");
                debug!("Applied migrations: {:?}", versions);
            }
            Ok(Self { pool })
        })
        .await
        .expect("database connection should never panic")
    }
    /// Fetch all blog posts from the database sorted by ID in descending order.
    /// Returns a `Vec<BlogPost>` if successful, or `DatabaseError::Sql` if the query fails.
    pub async fn fetch_all(&self) -> Result<Vec<BlogPost>, DatabaseError> {
        debug!("Loading blog posts");
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = pool.get()?;
            let result = blog_post
                .select(BlogPost::as_select())
                .order(id.desc())
                .load(&mut connection)?;
            Ok(result)
        })
        .await
        .expect("database query should never panic")
    }
    /// Save a new blog post to the database.
    /// Returns the saved `BlogPost` if successful, or `DatabaseError::Sql` if the query fails.
    pub async fn save(&self, to_persist: InsertBlogPost) -> Result<BlogPost, DatabaseError> {
        debug!("Saving blog post: {:?}", to_persist);
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = pool.get()?;
            let result = diesel::insert_into(blog_post)
                .values(&to_persist)
                .returning(BlogPost::as_returning())
                .get_result(&mut connection)?;
            Ok(result)
        })
        .await
        .expect("database query should never panic")
    }
    /// Delete a blog post from the database by ID.
    /// Returns the deleted `BlogPost` if successful, or `DatabaseError::Sql` if the query fails.
    pub async fn delete(&self, post_id: BlogPostId) -> Result<BlogPost, DatabaseError> {
        debug!("Deleting blog post with id: {}", post_id);
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = pool.get()?;
            let result = diesel::delete(blog_post.find(post_id))
                .returning(BlogPost::as_returning())
                .get_result(&mut connection)?;
            Ok(result)
        })
        .await
        .expect("database query should never panic")
    }
}
