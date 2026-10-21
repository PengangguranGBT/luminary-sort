//! Parallel directory scanning + metadata collection.

use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::metadata::{metadata_of, MediaError};
use crate::model::MediaFile;

pub struct IndexBuilder {
    pub files: Vec<MediaFile>,
}

impl IndexBuilder {
    /// Walks `root` and collects light metadata for every regular file.
    /// Files are walked once (sequentially) and parsed in parallel.
    pub fn build(root: &PathBuf) -> Result<Self, MediaError> {
        let entries = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();

        let files = entries
            .par_iter()
            .filter_map(|p| metadata_of(p).ok())
            .collect::<Vec<_>>();

        Ok(IndexBuilder { files })
    }
}
