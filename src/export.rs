//! Copy a selection of media files into an export folder with unique names.

use crate::model::MediaFile;
use std::fs;
use std::path::Path;

pub fn export_files(files: &[MediaFile], dest: &Path) -> Result<usize, std::io::Error> {
    fs::create_dir_all(dest)?;
    let mut written = 0;
    for (i, f) in files.iter().enumerate() {
        let stem = f
        .path
        .file_stem()
        .and_then(|e| e.to_str())
        .unwrap_or("file")
        .to_string();
        let ext = f
        .path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_string();
        let name = format!("{}_{:04}-{}.{}", i, i, stem, ext);
        fs::copy(&f.path, dest.join(name))?;
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MediaKind;
    use uuid::Uuid;

    #[test]
    fn export_writes_files_unique() {
        let dir = std::env::temp_dir().join("luminary_sort_export_test");
        let _ = std::fs::remove_dir_all(&dir);
        let src = std::env::temp_dir().join("a.jpg");
        std::fs::write(&src, b"payload").unwrap();
        let files = vec![MediaFile {
            id: Uuid::new_v4(),
            path: src,
            size: 100,
            modified: chrono::Utc::now(),
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
        }];
        let n = export_files(&files, &dir).unwrap();
        assert_eq!(n, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
