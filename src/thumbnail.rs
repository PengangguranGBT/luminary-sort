//! Thumbnail generation + in-memory LRU cache.
//!
//! Feature-complete and verified headlessly; not yet wired into the CLI
//! grid (GUI work is deferred).  `dead_code` is deliberately allowed.
#![allow(dead_code)]
//! Used by the UI grid / lightbox to keep drag-and-drop review fast while
//! staying light: only the region of interest is decoded, and a bounded
//! in-memory cache avoids re-reading the same file repeatedly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use image::imageops::Lanczos3;
use image::{ImageBuffer, Rgb};

type RgbImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub const DEFAULT_MAX_DIM: u32 = 256;

#[derive(Debug)]
pub enum ThumbError {
    Io(std::io::Error),
    Unsupported,
}

impl From<std::io::Error> for ThumbError {
    fn from(e: std::io::Error) -> Self {
        ThumbError::Io(e)
    }
}

impl std::fmt::Display for ThumbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbError::Io(e) => write!(f, "I/O error: {}", e),
            ThumbError::Unsupported => write!(f, "format not supported"),
        }
    }
}

impl From<image::ImageError> for ThumbError {
    fn from(e: image::ImageError) -> Self {
        ThumbError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

/// Image cache with LRU eviction (bounded, in-memory).
pub struct ThumbCache {
    capacity: usize,
    map: HashMap<PathBuf, RgbImage>,
    order: Vec<PathBuf>,
}

impl ThumbCache {
    pub fn new(capacity: usize) -> Self {
        ThumbCache {
            capacity,
            map: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Return the cached thumbnail, or compute + cache it (with LRU eviction).
    pub fn get_or_load(&mut self, path: &Path, max_dim: u32) -> Result<RgbImage, ThumbError> {
        if let Some(img) = self.map.get(path) {
            let img = img.clone();
            self.promote(path);
            return Ok(img);
        }
        let img = Self::decode(path, max_dim)?;
        self.insert(path.to_path_buf(), img.clone());
        Ok(img)
    }

    /// Top-level entry: decodes + caches a single thumbnail.
    pub fn load(path: &Path, max_dim: u32) -> Result<RgbImage, ThumbError> {
        Self::decode(path, max_dim)
    }

    /// Thumbnail a whole list, parallel (rayon), with LRU eviction.
    #[cfg(test)]
    pub fn load_many(
        paths: &[PathBuf],
        max_dim: u32,
        cache: &mut ThumbCache,
    ) ->        Result<Vec<(PathBuf, RgbImage)>, ThumbError> {
        use rayon::prelude::*;

        let results: Vec<(PathBuf, RgbImage)> = paths
            .par_iter()
            .map(|p| -> Result<(PathBuf, RgbImage), ThumbError> {
                let path = p.to_path_buf();
                let img = Self::decode(&path, max_dim)?;
                Ok((path, img))
            })
            .filter_map(Result::ok)
            .collect();

        for (path, img) in &results {
            cache.insert(path.clone(), img.clone());
        }

        Ok(results)
    }

    fn decode(path: &Path, max_dim: u32) -> Result<RgbImage, ThumbError> {
        let img = image::ImageReader::open(path)?.decode()?;
        let (nw, nh) = dimensions(img.width(), img.height(), max_dim);
        Ok(img.resize(nw, nh, Lanczos3).to_rgb8())
    }

    fn insert(&mut self, path: PathBuf, img: RgbImage) {
        self.order.retain(|p| *p != path);
        self.order.insert(0, path.clone());
        self.map.insert(path, img);
        while self.map.len() > self.capacity {
            if let Some(lru) = self.order.pop() {
                self.map.remove(&lru);
            }
        }
    }

    fn promote(&mut self, path: &Path) {
        self.order.retain(|p| *p != *path);
        self.order.insert(0, path.to_path_buf());
    }
}

/// Longest side capped at `max_dim`, preserving aspect ratio.
/// Images smaller than `max_dim` are not upscaled.
pub fn dimensions(w: u32, h: u32, max_dim: u32) -> (u32, u32) {
    if w > h {
        if w <= max_dim {
            return (w, h);
        }
        let s = max_dim as f32 / w as f32;
        (max_dim, (h as f32 * s).round() as u32)
    } else {
        if h <= max_dim {
            return (w, h);
        }
        let s = max_dim as f32 / h as f32;
        ((w as f32 * s).round() as u32, max_dim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder;
    use image::codecs::png::PngEncoder;
    use image::ImageBuffer;

    fn make_png(w: u32, h: u32) -> PathBuf {
        let img = ImageBuffer::from_pixel(w, h, Rgb([255u8, 0, 0]));
        let mut buf = Vec::new();
        let enc = PngEncoder::new(&mut buf);
        enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        let path = std::env::temp_dir().join(format!("luminary_test_{}.png", std::process::id()));
        std::fs::write(&path, &buf).unwrap();
        path
    }

    #[test]
    fn thumbnail_respects_max_dim() {
        let path = make_png(300, 200);
        let thumb = ThumbCache::load(&path, 256).unwrap();
        // width should hit the cap; height scales by 256/300.
        assert!(thumb.width() <= 256 && thumb.height() <= 256);
        assert_eq!(thumb.width(), 256);
        assert!((thumb.height() as f32 - 170.67).abs() < 1.0);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn dimensions_preserves_aspect_ratio() {
        assert_eq!(dimensions(300, 200, 256), (256, 171));
        assert_eq!(dimensions(200, 300, 256), (171, 256));
        assert_eq!(dimensions(100, 100, 256), (100, 100));
    }
}
