//! Image processing utilities for the server.

use crate::model::{AvatarImageUuid, PostImageUuid};
use image::{DynamicImage, ImageError, ImageFormat, ImageReader};
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::try_join;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

/// Errors that can occur when processing images.
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum AppImageError {
    #[display("Error downloading image: {}", _0)]
    Download(reqwest::Error),
    #[display("Error decoding image: {}", _0)]
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

/// The `ImageUuid` trait is used to abstract over the different types of images that can be saved.
pub trait ImageUuid: Debug + Send + 'static {
    fn new(uuid: Uuid) -> Self;
    fn path(&self) -> PathBuf;
}

impl ImageUuid for PostImageUuid {
    fn new(uuid: Uuid) -> Self {
        PostImageUuid {
            uuid: uuid.to_string(),
        }
    }

    /// Post images are stored in the `images/posts` directory.
    /// Returns the path to the image file on the file system.
    fn path(&self) -> PathBuf {
        image_path("posts", &self.uuid)
    }
}

impl ImageUuid for AvatarImageUuid {
    fn new(uuid: Uuid) -> Self {
        AvatarImageUuid {
            uuid: uuid.to_string(),
        }
    }

    /// Avatars are stored in the `images/avatars` directory.
    /// Returns the path to the image file on the file system.
    fn path(&self) -> PathBuf {
        image_path("avatars", &self.uuid)
    }
}

/// Preprocesses the post image bytes and avatar URL, if present.
/// Returns the UUIDs of the saved images, if any.
pub async fn process_images(
    post_image_bytes: Option<Vec<u8>>,
    avatar_url: Option<String>,
) -> Result<(Option<PostImageUuid>, Option<AvatarImageUuid>), AppImageError> {
    match (post_image_bytes, avatar_url) {
        (None, None) => {
            debug!("No images to process");
            Ok((None, None))
        }
        (Some(post_image), None) => {
            debug!("Processing post image");
            let image = process_image(post_image).await?;
            let image_uuid = save(image).await?;
            Ok((Some(image_uuid), None))
        }
        (None, Some(avatar_url)) => {
            debug!("Processing avatar image");
            let avatar = process_avatar(avatar_url).await?;
            let avatar_uuid = save(avatar).await?;
            Ok((None, Some(avatar_uuid)))
        }
        (Some(post_image), Some(avatar_url)) => {
            debug!("Processing post and avatar images");
            let (image, avatar) = try_join!(process_image(post_image), process_avatar(avatar_url))?;
            let (image_uuid, avatar_uuid) = try_join!(save(image), save(avatar))?;
            Ok((Some(image_uuid), Some(avatar_uuid)))
        }
    }
}

/// Validate that the bytes are a PNG image, if present.
async fn process_image(bytes: Vec<u8>) -> Result<DynamicImage, AppImageError> {
    let image = decode(bytes).await?;
    Ok(image)
}

/// Download the file at the URL and validate that it is a PNG image, if present.
async fn process_avatar(url: String) -> Result<DynamicImage, AppImageError> {
    let bytes = download(url).await?;
    let image = decode(bytes).await?;
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
#[instrument(skip(image))]
async fn save<I: ImageUuid>(image: DynamicImage) -> Result<I, AppImageError> {
    tokio::task::spawn_blocking(move || {
        let save = I::new(Uuid::new_v4());
        let path = save.path();
        // Create the directory if it doesn't exist
        // Safety: We know the parent directory exists because we are creating the path from the UUID
        std::fs::create_dir_all(path.parent().expect("parent dir should exist"))?;
        image.save(path)?;
        Ok(save)
    })
    .await
    .expect("saving should not panic")
    .inspect(|save| debug!("Saved image to {}", save.path().display()))
    .inspect_err(|e| warn!("Failed to save image: {}", e))
}

/// Loads an image from the file system if it exists.
#[instrument]
#[rustfmt::skip]
pub async fn load<I: ImageUuid>(image: &I) -> Result<Vec<u8>, AppImageError> {
    let path = image.path();
    tokio::task::spawn_blocking(move || std::fs::read(path))
        .await
        .expect("loading should not panic")
        .inspect(|_| debug!("Loaded image from {}", image.path().display()))
        .inspect_err(|e| warn!("Failed to load image from {}: {}", image.path().display(), e))
        .map_err(Into::into)
}

/// Deletes an image from the file system if it exists.
/// This function accepts an optional for convenience (see call site).
#[instrument]
pub async fn delete<I: ImageUuid>(image: Option<&I>) -> Result<(), AppImageError> {
    match image {
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
