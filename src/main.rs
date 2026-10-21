//! Entry point for luminary-sort.

mod collection;
mod export;
mod filter;
mod metadata;
mod model;
mod scan;
mod thumbnail;
mod gui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use collection::Selection;
use export::export_files;
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use model::{Index, MediaFile, MediaKind, Project};
use scan::IndexBuilder;
use thumbnail::ThumbCache;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "luminary-sort")]
#[command(about = "Fast, minimal black-and-white photo & video sorter")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Index a folder (build the media index)
    Import {
        root: PathBuf,
        #[arg(short, long)]
        index: Option<PathBuf>,
    },
    /// List media already indexed
    List { index: PathBuf },
    /// Save a filtered subset as a selection
    Select {
        index: PathBuf,
        out: PathBuf,
        #[arg(short, long)]
        tags: Option<String>,
        #[arg(short, long)]
        rating: Option<u8>,
        #[arg(short, long)]
        flagged: bool,
    },
    /// Export a saved selection to a folder
    Export { selection: PathBuf, dest: PathBuf },
    /// Thumbnails for every indexed media file
    Thumbnail {
        root: PathBuf,
        dest: PathBuf,
        #[arg(short, long, default_value_t = 256)]
        max_dim: u32,
    },
    /// Named projects (client/session folders)
    Project {
        index: PathBuf,
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Desktop GUI for the media index
    Gui { index: PathBuf },
}

/// Actions inside `luminary-sort project ...`
#[derive(Subcommand)]
enum ProjectAction {
    /// Create a named project (assigns media to a client/session)
    Add {
        name: String,
        #[arg(short, long)]
        tags: Option<String>,
        #[arg(short, long)]
        rating: Option<u8>,
        #[arg(short, long)]
        flagged: bool,
    },
    /// List projects
    List,
    /// Remove a project
    Remove { name: String },
    /// Export a project's media files to a folder
    Export { name: String, dest: PathBuf },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Import { root, index } => {
            let builder = IndexBuilder::build(&root)?;
            let idx = Index { files: builder.files, projects: Vec::new() };

            if idx.files.is_empty() {
                eprintln!("No media files found under {}", root.display());
                return Ok(());
            }

            let images = idx.files.iter().filter(|f| f.kind == MediaKind::Image).count();
            let videos = idx.files.iter().filter(|f| f.kind == MediaKind::Video).count();
            let other = idx.files.len() - images - videos;
            println!(
                "Indexed {} media files ({} images, {} videos, {} other) under {}",
                idx.files.len(),
                images,
                videos,
                other,
                root.display()
            );

            if let Some(ref p) = index {
                let data = ron::to_string(&idx)?;
                std::fs::write(p, data)?;
                println!("Index written to {}", p.display());
            }
            Ok(())
        }
        Command::List { index } => {
            let data = std::fs::read(&index)?;
            let idx: Index = ron::from_str(&String::from_utf8_lossy(&data))?;
            for f in &idx.files {
                println!(
                    "{:<14} {:>6}  {}  {}",
                    f.path.display(),
                    f.size,
                    f.kind.to_string(),
                    f.path.file_name().unwrap().to_string_lossy()
                );
            }
            Ok(())
        }
        Command::Select {
            index: idx_path,
            out,
            tags,
            rating,
            flagged,
        } => {
            let data = std::fs::read(&idx_path)?;
            let idx: Index = ron::from_str(&String::from_utf8_lossy(&data))?;
            let mut sel = Selection::new("selection");
            if let Some(t) = tags {
                sel.tags = t
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            if let Some(r) = rating {
                sel.min_rating = r;
            }
            sel.flagged = flagged;

            let selected = sel.select(&idx);
            let data = ron::to_string(&selected)?;
            std::fs::write(&out, data)?;
            println!("Selection '{}' -> {} files", sel.name, selected.len());
            Ok(())
        }
        Command::Export { selection, dest } => {
            let data = std::fs::read(&selection)?;
            let files: Vec<MediaFile> = ron::from_str(&String::from_utf8_lossy(&data))?;
            let n = export_files(&files, &dest)?;
            println!("Exported {} files to {}", n, dest.display());
            Ok(())
        }
        Command::Thumbnail { root, dest, max_dim } => {
            let builder = IndexBuilder::build(&root)?;
            let idx = Index { files: builder.files, projects: Vec::new() };
            if idx.files.is_empty() {
                eprintln!("No media files found under {}", root.display());
                return Ok(());
            }
            let mut cache = ThumbCache::new(128);
            let mut written = 0;
            for (i, f) in idx.files.iter().enumerate() {
                match cache.get_or_load(&f.path, max_dim) {
                    Ok(thumb) => {
                        let ext = f.path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("jpg");
                        let name = format!("thumb_{:04}.{}", i, ext.to_lowercase());
                        let out_path = dest.join(&name);
                        {
                            let file = std::fs::File::create(&out_path)?;
                            let mut buf = std::io::BufWriter::new(file);
                            let enc = PngEncoder::new(&mut buf);
                            enc.write_image(
                                thumb.as_raw(),
                                thumb.width(),
                                thumb.height(),
                                image::ExtendedColorType::Rgb8,
                            )?;
                        }
                        written += 1;
                    }
                    Err(e) => eprintln!(
                        "Thumbnail failed for {}: {}",
                        f.path.display(),
                        e
                    ),
                }
            }
            println!("Thumbnails: {} written to {}", written, dest.display());
            Ok(())
        }
        Command::Project { index, action } => {
            let data = std::fs::read(&index)?;
            let mut idx: Index = ron::from_str(&String::from_utf8_lossy(&data))?;
            match action {
                ProjectAction::Add { name, tags, rating, flagged } => {
                    let mut proj = Project {
                        id: Uuid::new_v4(),
                        name: name.clone(),
                        media_ids: Vec::new(),
                    };
                    for f in &idx.files {
                        let mut include = true;
                        if let Some(t) = tags.as_deref() {
                            let req: Vec<&str> = t
                                .split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .collect();
                            if !req.is_empty() && !req.iter().all(|r| f.tags.iter().any(|x| x == r)) {
                                include = false;
                            }
                        }
                        if include {
                            if let Some(r) = rating {
                                if f.rating < r {
                                    include = false;
                                }
                            }
                        }
                        if include {
                            if f.flagged != flagged {
                                include = false;
                            }
                        }
                        if include {
                            proj.media_ids.push(f.id);
                        }
                    }
                    let n_files = proj.media_ids.len();
                    idx.projects.push(proj);
                    let data = ron::to_string(&idx)?;
                    std::fs::write(&index, data)?;
                    println!("Project '{}' -> {} file(s)", name, n_files);
                }
                ProjectAction::List => {
                    for p in &idx.projects {
                        println!("  {}  ({})", p.name, p.media_ids.len());
                    }
                }
                ProjectAction::Remove { name } => {
                    idx.projects.retain(|p| p.name != name);
                    let data = ron::to_string(&idx)?;
                    std::fs::write(&index, data)?;
                    println!("Removed project '{}'", name);
                }
                ProjectAction::Export { name, dest } => {
                    let proj = idx
                        .projects
                        .iter()
                        .find(|p| p.name == name)
                        .ok_or(format!("Project '{}' not found", name))?;
                    let media: Vec<MediaFile> = idx
                        .files
                        .iter()
                        .filter(|f| proj.media_ids.contains(&f.id))
                        .cloned()
                        .collect();
                    let n = export_files(&media, &dest)?;
                    println!("Exported {} files to {}", n, dest.display());
                }
            }
            Ok(())
        }
    Command::Gui { index } => {
            eframe::run_native(
                "luminary-sort",
                eframe::NativeOptions::default(),
                Box::new(|_cc| Box::new(gui::SortApp::new(Some(index)))),
            )?;
            Ok(())
        }
}
}
