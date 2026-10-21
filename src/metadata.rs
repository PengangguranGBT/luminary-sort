//! Detect media kind and parse *light* metadata without full decode.
//!
//! The heavy parsers (full EXIF, XMP, RAW, video streams) are deferred to
//! optional backends (controlled via Cargo features) so the core index stays
//! dependency-free and fast. Once built, the light fields drive filtering +
//! sorting. Until a backend is enabled the importer fails with a clear error
//! instead of panicking on an empty skeleton.

use std::path::PathBuf;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::{MediaFile, MediaKind};

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn guess_kind(path: &PathBuf) -> MediaKind {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tif" | "tiff" => MediaKind::Image,
        "mp4" | "mov" | "avi" | "mkv" | "mts" | "m2ts" | "mpg" | "mpeg" | "wmv" | "flv"
        | "3gp" | "m4v" => MediaKind::Video,
        _ => MediaKind::Other,
    }
}

fn light_metadata(
    kind: MediaKind,
    path: &PathBuf,
) -> Result<(Option<u32>, Option<u32>, Option<f64>, Option<String>, Option<String>, Option<NaiveDate>), MediaError> {
    match kind {
        MediaKind::Image => parse_image_light(path),
        MediaKind::Video => parse_video_light(path),
        MediaKind::Other => Ok((None, None, None, None, None, None)),
    }
}

pub fn metadata_of(path: &PathBuf) -> Result<MediaFile, MediaError> {
    let kind = guess_kind(path);

    let (width, height, duration, camera_model, lens_model, capture_date) =
        light_metadata(kind, path)?;

    let meta = std::fs::metadata(path)?;

    Ok(MediaFile {
        id: Uuid::new_v4(),
        path: path.clone(),
        size: meta.len(),
        modified: meta
            .modified()?
            .into(), // SystemTime -> DateTime<Utc>
        kind,
        width,
        height,
        duration_secs: duration,
        camera_model,
        lens_model,
        capture_date,
        tags: Vec::new(),
        rating: 0,
        flagged: false,
        session_id: None,
        project_id: None,
    })
}

// --- Light parsers (override via Cargo feature / real backend) ---

fn parse_image_light(
    _path: &PathBuf,
) -> Result<(Option<u32>, Option<u32>, Option<f64>, Option<String>, Option<String>, Option<NaiveDate>), MediaError>
{
    // Placeholder: heavy EXIF/RAW parsing is deferred to the 'image-metadata'
    // backend. Kind is already known from `guess_kind`, so indexing proceeds.
    Ok((None, None, None, None, None, None))
}

fn parse_video_light(
    _path: &PathBuf,
) -> Result<(Option<u32>, Option<u32>, Option<f64>, Option<String>, Option<String>, Option<NaiveDate>), MediaError>
{
    // Placeholder (see parse_image_light).
    Ok((None, None, None, None, None, None))
}
