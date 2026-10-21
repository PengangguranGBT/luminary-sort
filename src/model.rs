//! Domain model: media records and the index container.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MediaKind {
    Image,
    Video,
    Other,
}

impl std::fmt::Display for MediaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaKind::Image => write!(f, "image"),
            MediaKind::Video => write!(f, "video"),
            MediaKind::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: Uuid,
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub kind: MediaKind,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub capture_date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub rating: u8,
    pub flagged: bool,
    pub session_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    /// Human-readable project name (e.g. client name or session).
    pub name: String,
    /// Media file IDs belonging to this project.
    pub media_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub files: Vec<MediaFile>,
    /// Persisted named projects (client/session folders). Defaults to empty for
    /// backward compatibility with older `.lumi` files.
    #[serde(default)]
    pub projects: Vec<Project>,
}
