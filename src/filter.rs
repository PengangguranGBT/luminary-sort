//! Filter + sort engine over the indexed media set.
//!
//! Feature-complete and verified headlessly; sort CLI wiring (Select/Export)
//! is deferred to the GUI milestone.  `dead_code` is deliberately allowed.
#![allow(dead_code)]

use chrono::NaiveDate;

use crate::model::{Index, MediaFile, MediaKind};

/// Query used to select a subset of media by kind, date range, camera model,
/// tags (AND), minimum rating, and flag status.
#[derive(Debug, Clone)]
pub struct FilterQuery {
    pub kinds: Option<Vec<MediaKind>>,
    pub capture_date_min: Option<NaiveDate>,
    pub capture_date_max: Option<NaiveDate>,
    pub camera_model: Option<String>,
    pub tags: Vec<String>,
    pub min_rating: u8,
    pub flagged: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum SortField {
    CaptureDate,
    Name,
    Size,
    Rating,
    Duration,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Index {
    /// Filter + multi-tag AND select over the indexed media.
    pub fn filter(&self, q: &FilterQuery) -> Vec<MediaFile> {
        self.files
            .iter()
            .filter(|f| {
                if let Some(kinds) = &q.kinds {
                    if !kinds.iter().any(|k| k == &f.kind) {
                        return false;
                    }
                }
                if let Some(min) = q.capture_date_min {
                    if let Some(d) = f.capture_date {
                        if d < min {
                            return false;
                        }
                    }
                }
                if let Some(max) = q.capture_date_max {
                    if let Some(d) = f.capture_date {
                        if d > max {
                            return false;
                        }
                    }
                }
                if let Some(model) = &q.camera_model {
                    if f.camera_model.as_deref() != Some(model) {
                        return false;
                    }
                }
                for tag in &q.tags {
                    if !f.tags.contains(tag) {
                        return false;
                    }
                }
                if q.min_rating > 0 && f.rating < q.min_rating {
                    return false;
                }
                if let Some(flagged) = q.flagged {
                    if f.flagged != flagged {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn sort(&mut self, field: SortField, order: SortOrder) {
        self.files.sort_by(|a, b| {
            let result = match field {
                SortField::CaptureDate => a
                    .capture_date
                    .cmp(&b.capture_date),
                SortField::Name => a
                    .path
                    .file_name()
                    .cmp(&b.path.file_name()),
                SortField::Size => a.size.cmp(&b.size),
                SortField::Rating => a.rating.cmp(&b.rating),
                SortField::Duration => compare_duration(&a.duration_secs, &b.duration_secs),
            };
            match order {
                SortOrder::Asc => result,
                SortOrder::Desc => result.reverse(),
            }
        });
    }
}

fn compare_duration(a: &Option<f64>, b: &Option<f64>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(x), Some(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn media(path: &str, size: u64, kind: MediaKind, tags: &[&str]) -> MediaFile {
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
    fn filter_intersects_tags() {
        let index = Index { projects: Vec::new(),
            files: vec![
                media("a.jpg", 10, MediaKind::Image, &["urban", "night"]),
                media("b.jpg", 20, MediaKind::Image, &["urban", "day"]),
                media("c.jpg", 30, MediaKind::Image, &["portrait"]),
            ],
        };
        let q = FilterQuery {
            kinds: None,
            capture_date_min: None,
            capture_date_max: None,
            camera_model: None,
            tags: vec!["urban".into(), "night".into()],
            min_rating: 0,
            flagged: None,
        };
        let out = index.filter(&q);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn filter_by_flagged() {
        let index = Index { projects: Vec::new(),
            files: vec![
                MediaFile {
                    id: Uuid::new_v4(),
                    path: PathBuf::from("a.jpg"),
                    size: 10,
                    modified: Utc::now(),
                    kind: MediaKind::Image,
                    width: None,
                    height: None,
                    duration_secs: None,
                    camera_model: None,
                    lens_model: None,
                    capture_date: None,
                    tags: vec![],
                    rating: 0,
                    flagged: true,
                    session_id: None,
                    project_id: None,
                },
                MediaFile {
                    id: Uuid::new_v4(),
                    path: PathBuf::from("b.jpg"),
                    size: 20,
                    modified: Utc::now(),
                    kind: MediaKind::Image,
                    width: None,
                    height: None,
                    duration_secs: None,
                    camera_model: None,
                    lens_model: None,
                    capture_date: None,
                    tags: vec![],
                    rating: 0,
                    flagged: false,
                    session_id: None,
                    project_id: None,
                },
            ],
        };
        let q = FilterQuery {
            kinds: None,
            capture_date_min: None,
            capture_date_max: None,
            camera_model: None,
            tags: vec![],
            min_rating: 0,
            flagged: Some(true),
        };
        let out = index.filter(&q);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].path, PathBuf::from("a.jpg"));
    }

    #[test]
    fn filter_by_rating() {
        let index = Index { projects: Vec::new(),
            files: vec![
                MediaFile {
                    id: Uuid::new_v4(),
                    path: PathBuf::from("a.jpg"),
                    size: 10,
                    modified: Utc::now(),
                    kind: MediaKind::Image,
                    width: None,
                    height: None,
                    duration_secs: None,
                    camera_model: None,
                    lens_model: None,
                    capture_date: None,
                    tags: vec![],
                    rating: 1,
                    flagged: false,
                    session_id: None,
                    project_id: None,
                },
                MediaFile {
                    id: Uuid::new_v4(),
                    path: PathBuf::from("b.jpg"),
                    size: 20,
                    modified: Utc::now(),
                    kind: MediaKind::Image,
                    width: None,
                    height: None,
                    duration_secs: None,
                    camera_model: None,
                    lens_model: None,
                    capture_date: None,
                    tags: vec![],
                    rating: 2,
                    flagged: false,
                    session_id: None,
                    project_id: None,
                },
                MediaFile {
                    id: Uuid::new_v4(),
                    path: PathBuf::from("c.jpg"),
                    size: 30,
                    modified: Utc::now(),
                    kind: MediaKind::Image,
                    width: None,
                    height: None,
                    duration_secs: None,
                    camera_model: None,
                    lens_model: None,
                    capture_date: None,
                    tags: vec![],
                    rating: 5,
                    flagged: false,
                    session_id: None,
                    project_id: None,
                },
            ],
        };
        let q = FilterQuery {
            kinds: None,
            capture_date_min: None,
            capture_date_max: None,
            camera_model: None,
            tags: vec![],
            min_rating: 3,
            flagged: None,
        };
        let out = index.filter(&q);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].size, 30);
    }

    #[test]
    fn sort_by_size_desc() {
        let mut index = Index { projects: Vec::new(),
            files: vec![
                media("a.jpg", 100, MediaKind::Image, &[]),
                media("b.jpg", 50, MediaKind::Image, &[]),
                media("c.jpg", 200, MediaKind::Image, &[]),
            ],
        };
        index.sort(SortField::Size, SortOrder::Desc);
        assert_eq!(index.files[0].size, 200);
        assert_eq!(index.files[2].size, 50);
    }
}
