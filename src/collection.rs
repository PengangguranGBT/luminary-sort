//! Selection: a saved subset of an index (filter + sort), persisted to disk.
//!
//! Test helpers require `PathBuf`/`Utc`/`Uuid`; the non-test build leaves them
//! unused, so `dead_code` and `unused_imports` are deliberately allowed.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;

use crate::filter::FilterQuery;
use crate::model::{Index, MediaFile, MediaKind};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub name: String,
    pub kinds: Option<Vec<MediaKind>>,
    pub tags: Vec<String>,
    pub min_rating: u8,
    pub flagged: bool,
}

impl Selection {
    pub fn new(name: &str) -> Self {
        Selection {
            name: name.to_string(),
            kinds: None,
            tags: Vec::new(),
            min_rating: 0,
            flagged: false,
        }
    }

    pub fn select(&self, index: &Index) -> Vec<MediaFile> {
        let q = FilterQuery {
            kinds: self.kinds.clone(),
            capture_date_min: None,
            capture_date_max: None,
            camera_model: None,
            tags: self.tags.clone(),
            min_rating: self.min_rating,
            flagged: Some(self.flagged),
        };
        index.filter(&q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MediaKind;

    fn media_file(path: &str, size: u64, kind: MediaKind, tags: &[&str]) -> MediaFile {
        MediaFile {
            id: Uuid::new_v4(),
            path: PathBuf::from(path),
            size,
            modified: Utc::now(),
            kind,
            width: None,
            height: None,
            duration_secs: None,
            camera_model: None,
            lens_model: None,
            capture_date: None,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            rating: 0,
            flagged: false,
            session_id: None,
            project_id: None,
        }
    }

    #[test]
    fn selection_filters_by_tags() {
        let index = Index { projects: Vec::new(),
            files: vec![
                media_file("a.jpg", 10, MediaKind::Image, &["urban", "night"]),
                media_file("b.jpg", 20, MediaKind::Image, &["urban", "day"]),
                media_file("c.jpg", 30, MediaKind::Image, &["portrait"]),
            ],
        };
        let sel = Selection {
            name: "t".into(),
            kinds: None,
            tags: vec!["urban".into(), "night".into()],
            min_rating: 0,
            flagged: false,
        };
        let out = sel.select(&index);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, PathBuf::from("a.jpg"));
    }
}
