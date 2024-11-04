//! Image processing utilities for the server.

use crate::model::{AvatarImagePath, PostImagePath};
use image::{DynamicImage, ImageError, ImageFormat, ImageReader};
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::try_join;
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

/// Errors that can occur when processing images.
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum AppImageError {
    #[display("Download error: {}", _0)]
    Download(reqwest::Error),
    #[display("Image error: {}", _0)]
    Decode(ImageError),
    #[display("IO error: {}", _0)]
    Io(std::io::Error),
}

/// Returns the path to the image with the provided UUID on the file system.
/// In order to prevent the file system from becoming overwhelmed,
/// images are stored in directories based on their type and the first four characters of their UUID.
/// Their file name is their UUID with a `.png` extension.
/// For example, a post image with UUID `123e4567-e89b-12d3-a456-426614174000` would be stored at:
/// `./images/posts/12/3e/123e4567-e89b-12d3-a456-426614174000.png`
///
/// Safety: Only call this function with valid UUIDs.
/// It will panic if there are not enough characters in the UUID.
fn image_path(dir: &str, uuid: &str) -> PathBuf {
    format!(
        "./images/{}/{}/{}/{}.png",
        dir,
        &uuid[0..2],
        &uuid[2..4],
        uuid
    )
    .into()
}

/// The `ImagePath` trait is used to abstract over the different locations where images are stored.
pub trait ImagePath: Debug + Send + 'static {
    fn new(uuid: Uuid) -> Self;
    fn path(&self) -> PathBuf;
}

impl ImagePath for PostImagePath {
    fn new(uuid: Uuid) -> Self {
        PostImagePath(uuid.to_string())
    }

    /// Post images are stored in the `images/posts` directory.
    /// Returns the path to the image file on the file system.
    fn path(&self) -> PathBuf {
        image_path("posts", &self.0)
    }
}

impl ImagePath for AvatarImagePath {
    fn new(uuid: Uuid) -> Self {
        AvatarImagePath(uuid.to_string())
    }

    /// Avatars are stored in the `images/avatars` directory.
    /// Returns the path to the image file on the file system.
    fn path(&self) -> PathBuf {
        image_path("avatars", &self.0)
    }
}

/// Preprocesses the post image bytes and avatar URL, if present.
/// Returns the UUIDs of the saved images, if any.
pub async fn process_images(
    post_image_bytes: Option<Vec<u8>>,
    avatar_url: Option<String>,
) -> Result<(Option<PostImagePath>, Option<AvatarImagePath>), AppImageError> {
    match (post_image_bytes, avatar_url) {
        (None, None) => {
            trace!("No images to process");
            Ok((None, None))
        }
        (Some(post_image), None) => {
            trace!("Processing post image");
            let image = process_image(post_image).await?;
            let image_path = save(image).await?;
            Ok((Some(image_path), None))
        }
        (None, Some(avatar_url)) => {
            trace!("Processing avatar image");
            let avatar = process_avatar(avatar_url).await?;
            let avatar_path = save(avatar).await?;
            Ok((None, Some(avatar_path)))
        }
        (Some(post_image), Some(avatar_url)) => {
            trace!("Processing post and avatar images");
            let (image, avatar) = try_join!(process_image(post_image), process_avatar(avatar_url))?;
            let (image_path, avatar_path) = try_join!(save(image), save(avatar))?;
            Ok((Some(image_path), Some(avatar_path)))
        }
    }
}

/// Validate that the bytes are a PNG image, if present.
async fn process_image(bytes: Vec<u8>) -> Result<DynamicImage, AppImageError> {
    let image = decode(bytes).await?;
    // Do more processing here if needed, e.g. resizing
    Ok(image)
}

/// Download the file at the URL and validate that it is a PNG image, if present.
async fn process_avatar(url: String) -> Result<DynamicImage, AppImageError> {
    let bytes = download(url).await?;
    let image = decode(bytes).await?;
    // Do more processing here if needed, e.g. resizing
    Ok(image)
}

/// Downloads the bytes at the provided URL.
async fn download(url: String) -> Result<Vec<u8>, reqwest::Error> {
    debug!("Downloading image from {}", url);
    reqwest::get(&url)
        .await?
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
}

/// Validates that the provided bytes are a PNG image.
/// Returns the decoded image if it is a PNG, otherwise returns an error.
async fn decode(image_bytes: Vec<u8>) -> Result<DynamicImage, ImageError> {
    ImageReader::with_format(std::io::Cursor::new(image_bytes), ImageFormat::Png).decode()
}

/// Save the image to the file system.
/// This creates a new UUID for the image, saves the image to the corresponding file path,
/// and returns the UUID in the corresponding newtype.
#[instrument(skip(image))]
async fn save<Path: ImagePath>(image: DynamicImage) -> Result<Path, AppImageError> {
    tokio::task::spawn_blocking(move || {
        let image_path = Path::new(Uuid::new_v4());
        let path = image_path.path();
        // Create the directory if it doesn't exist
        // Safety: We know the parent directory exists because we are creating the path from the UUID
        std::fs::create_dir_all(path.parent().expect("parent dir should exist"))?;
        image.save(path)?;
        Ok(image_path)
    })
    .await
    .expect("saving should not panic")
    .inspect(|save| debug!("Saved image to {}", save.path().display()))
    .inspect_err(|e| warn!("Failed to save image: {}", e))
}

/// Loads the image from the file system with the provided UUID.
#[instrument]
#[rustfmt::skip]
pub async fn load<I: ImagePath>(image_uuid: &I) -> Result<Vec<u8>, AppImageError> {
    let path = image_uuid.path();
    tokio::task::spawn_blocking(move || std::fs::read(path))
        .await
        .expect("loading should not panic")
        .inspect(|_| trace!("Loaded image from {}", image_uuid.path().display()))
        .inspect_err(|e| warn!("Failed to load image from {}: {}", image_uuid.path().display(), e))
        .map_err(Into::into)
}

/// Deletes an image from the file system if it exists.
/// This function accepts an optional for convenience (see call site).
#[instrument]
pub async fn delete<I: ImagePath>(image_uuid: Option<&I>) -> Result<(), AppImageError> {
    match image_uuid {
        None => Ok(()),
        #[rustfmt::skip]
        Some(image) => {
            let path = image.path();
            tokio::task::spawn_blocking(move || std::fs::remove_file(path))
                .await
                .expect("deleting should not panic")
                .inspect(|_| debug!("Deleted image from {}", image.path().display()))
                .inspect_err(|e| warn!("Failed to delete image from {}: {}", image.path().display(), e))
                .map_err(Into::into)
        }
    }
}
