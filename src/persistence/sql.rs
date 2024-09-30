use std::error::Error;
use crate::persistence::model::{BlogPost, BlogPostCreateRequest, InsertBlogPost};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, info, trace};
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum DatabaseError {
    #[display("Connection error: {}", _0)]
    Connection(r2d2::Error),
    #[display("Migration error: {}", _0)]
    Migration(Box<dyn Error + Send + Sync>),
    #[display("SQL error: {}", _0)]
    Sql(diesel::result::Error),
}

impl Error for DatabaseError {}

pub(crate) struct Database(r2d2::Pool<ConnectionManager<SqliteConnection>>);

impl Database {
    /// Attempt to connect to the SQLite database at the provided URL.
    /// Create a connection pool and immediately run embedded Diesel migrations
    /// to ensure the schema is up-to-date.
    pub(crate) async fn try_connect(url: &str) -> Result<Self, DatabaseError> {
        debug!("Using database URL: {}", url);
        let pool = r2d2::Pool::builder()
            .max_size(5)
            .build(ConnectionManager::<SqliteConnection>::new(url))
            .map_err(DatabaseError::Connection)?;
        let conn = &mut pool.get().map_err(DatabaseError::Connection)?;
        info!("Connected to database.");
        let versions = conn.run_pending_migrations(MIGRATIONS).map_err(DatabaseError::Migration)?;
        if !versions.is_empty() {
            info!("Successfully updated database schema.");
            debug!("Applied migrations: {:?}", versions);
        }
        Ok(Self(pool))
    }
    fn connect(&self) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, DatabaseError> {
        self.0.get().map_err(DatabaseError::Connection)
    }
    pub(crate) async fn save(&self, post: BlogPostCreateRequest) -> Result<BlogPost, DatabaseError> {
        use crate::persistence::schema::blog_post::dsl::blog_post;
        trace!("Saving blog post: {:?}", post);
        let values = InsertBlogPost {
            posted_on: time::OffsetDateTime::now_utc().date(),
            text: post.text,
            username: post.username,
            image_uuid: post.image_url.map(|_| Uuid::new_v4().to_string()),
            avatar_uuid: post.avatar_url.map(|_| Uuid::new_v4().to_string())
        };
        trace!("Inserting values: {:?}", values);
        let mut connection = self.connect()?;
        tokio::task::spawn_blocking(move || {
            diesel::insert_into(blog_post)
                .values(&values)
                .returning(BlogPost::as_returning())
                .get_result(&mut connection)
                .map_err(DatabaseError::Sql)
        })
            .await
            .unwrap() // Diesel shouldn't ever panic
    }
    pub(crate) async fn fetch_all(&self) -> Result<Vec<BlogPost>, DatabaseError> {
        use crate::persistence::schema::blog_post::dsl::blog_post;
        trace!("Loading blog posts");
        let mut connection = self.connect()?;
        tokio::task::spawn_blocking(move || {
            blog_post
                .select(BlogPost::as_select())
                .load(&mut connection)
                .map_err(DatabaseError::Sql)
        })
            .await
            .unwrap() // Diesel shouldn't ever panic
    }
}