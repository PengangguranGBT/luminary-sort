//! egui-based desktop UI for luminary-sort.
//!
//! Loads an index (`.lumi`), renders a minimal black-and-white file grid,
//! and supports selection + lightbox. Designed to be pure logic + egui widgets;
//! verified as far as `cargo build` (headless).

use crate::export::export_files;
use crate::model::{Index, MediaFile, Project};
use crate::thumbnail::ThumbCache;
use eframe::egui;
use eframe::Frame;
use egui::Context;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

/// Desktop shell of `luminary-sort`.
pub struct SortApp {
    pub index: Option<Index>,
    pub selected_ids: HashSet<Uuid>,
    /// Last single-clicked id (for shift+click range selection)
    last_clicked_id: Option<Uuid>,
    /// Currently selected project for viewing/exporting
    pub selected_project_id: Option<Uuid>,
    /// Buffer for new project name input
    pub new_project_name: String,
    /// Buffer for project export destination
    pub export_dest: String,
    /// Path of currently loaded index (for auto-save)
    pub current_index_path: Option<PathBuf>,
    /// Status message shown to user (export success/error, import count, etc.)
    pub status_message: Option<String>,
    pub filter_tags: Option<String>,
    /// Preview (lightbox) id when mouse-overed file is being previewed.
    preview_id: Option<Uuid>,
}

impl SortApp {
    pub fn new(index_path: Option<PathBuf>) -> Self {
        let mut app = SortApp {
            index: None,
            selected_ids: HashSet::new(),
            last_clicked_id: None,
            selected_project_id: None,
            new_project_name: String::new(),
            export_dest: String::new(),
            current_index_path: index_path.clone(),
            status_message: None,
            filter_tags: None,
            preview_id: None,
        };
        if let Some(p) = index_path {
            let _ = app.load_index(p);
        }
        app
    }

    pub fn load_index(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(&path)?;
        if let Ok(idx) = ron::from_str::<Index>(&String::from_utf8_lossy(&data)) {
            self.index = Some(idx);
            self.current_index_path = Some(path);
        }
        Ok(())
    }

    pub fn save_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(ref idx) = self.index else { return Ok(()); };
        let Some(ref path) = self.current_index_path else { return Ok(()); };
        let data = ron::to_string(idx)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn reload_index(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(&path)?;
        if let Ok(idx) = ron::from_str::<Index>(&String::from_utf8_lossy(&data)) {
            self.index = Some(idx);
        }
        Ok(())
    }

    pub fn selected_files(&self) -> Vec<MediaFile> {
        if let Some(ref idx) = self.index {
            if !self.selected_ids.is_empty() {
                idx.files
                .iter()
                .filter(|f| self.selected_ids.contains(&f.id))
                .cloned()
                .collect()
            } else {
                // No selection: export all files
                idx.files.iter().cloned().collect()
            }
        } else {
            Vec::new()
        }
    }

    #[allow(dead_code)]
    fn filtered_files(&self) -> Vec<&MediaFile> {
        self.index
        .as_ref()
        .map(|idx| {
            idx.files
            .iter()
            .filter(|f| {
                if let Some(t) = &self.filter_tags {
                    let q: Vec<&str> = t
                    .split_whitespace()
                    .filter(|s| !s.is_empty())
                    .collect();
                    if q.is_empty() {
                        return true;
                    }
                    let mut matched = false;
                    for q in &q {
                        let q_lower = q.to_lowercase();
                        if f.path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_lowercase().contains(&q_lower))
                            .unwrap_or(false)
                            {
                                matched = true;
                            }
                            for tag in &f.tags {
                                if tag.to_lowercase().contains(&q_lower) {
                                    matched = true;
                                }
                            }
                    }
                    return matched;
                }
                true
            })
            .collect()
        })
        .unwrap_or_default()
    }

    fn thumb_color_image(&self, f: &MediaFile, max_dim: u32) -> Option<egui::ColorImage> {
        let img = ThumbCache::load(&f.path, max_dim).ok()?;
        let (w, h) = img.dimensions();
        let rgba: Vec<u8> = img
        .pixels()
        .flat_map(|p| {
            let b = p.0;
            [b[0], b[1], b[2], 255]
        })
        .collect();
        Some(egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba,
        ))
    }
}

impl eframe::App for SortApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // ---- top bar ----
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("Import", |ui| {
                        ui.label("Index file path:");
                        let mut import_path = String::new();
                        ui.text_edit_singleline(&mut import_path);
                        if ui.button("Load index").clicked() {
                            let path = std::path::PathBuf::from(import_path.trim());
                            match self.load_index(path.clone()) {
                                Ok(()) => {
                                    self.status_message = Some(format!("Index loaded: {}", path.display()));
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("Import error: {}", e));
                                }
                            }
                        }
                    });
                    ui.menu_button("Export", |ui| {
                        ui.label("Destination:");
                        let mut dest = String::new();
                        ui.text_edit_singleline(&mut dest);
                        ui.add_space(4.0);
                        if ui.button("Export selected").clicked() {
                            if let Err(e) = export_files(&self.selected_files(), std::path::Path::new(dest.trim())) {
                                self.status_message = Some(format!("Export error: {}", e));
                            } else {
                                self.status_message = Some("Exported successfully.".to_string());
                            }
                        }
                    });
                    ui.menu_button("Exit", |ui| {
                        ui.close_menu();
                    });
                });
            });
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.heading("luminary-sort");
                ui.label("minimal black & white photo + video sorter");
            });
        });

        // ---- status bar ----
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(msg) = &self.status_message {
                    ui.colored_label(egui::Color32::from_gray(150), msg);
                    if ui.button("✕").clicked() {
                        self.status_message = None;
                    }
                } else {
                    ui.label("");
                }
            });
        });

        // ---- central: file grid ----
        egui::CentralPanel::default().show(ctx, |ui| {
            // Rating/flagged change intents: (file_id, new_rating, new_flagged)
            let mut rating_changes: Vec<(Uuid, u8)> = Vec::new();
            let mut flagged_changes: Vec<(Uuid, bool)> = Vec::new();

            // Project intents (declared here so both ScrollArea UI and mutation block can access)
            let mut create_project = false;
            let mut new_project_name_buf = String::new();
            let mut remove_project_id: Option<Uuid> = None;
            let mut select_project_id: Option<Uuid> = None;
            let mut deselect_project = false;
            let mut export_project = false;
            let mut do_export_project = false;

            // toolbar
            egui::TopBottomPanel::bottom("toolbar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let search_text = self.filter_tags.get_or_insert_with(String::new);
                    ui.add(egui::TextEdit::singleline(search_text).hint_text("name / tags"));
                    ui.add_space(8.0);
                    if let Some(ref idx) = self.index {
                        ui.label(format!("{} files", idx.files.len()));
                    }
                });
            });

            // ---- filtered list ----
            egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Collect filtered IDs (owned, no borrow of self)
                let filtered_ids: Vec<Uuid> = {
                    if let Some(ref idx) = self.index {
                        idx.files
                        .iter()
                        .filter(|f| {
                            if let Some(t) = &self.filter_tags {
                                let q: Vec<&str> = t
                                .split_whitespace()
                                .filter(|s| !s.is_empty())
                                .collect();
                                if q.is_empty() { return true; }
                                let mut matched = false;
                                for q in &q {
                                    let q_lower = q.to_lowercase();
                                    if f.path
                                        .file_name()
                                        .map(|s| s.to_string_lossy().to_lowercase().contains(&q_lower))
                                        .unwrap_or(false)
                                    {
                                        matched = true;
                                    }
                                    for tag in &f.tags {
                                        if tag.to_lowercase().contains(&q_lower) {
                                            matched = true;
                                        }
                                    }
                                }
                                matched
                            } else {
                                true
                            }
                        })
                        .map(|f| f.id)
                        .collect()
                    } else {
                        Vec::new()
                    }
                };

            if filtered_ids.is_empty() {
                    ui.label("No matches found.");
                } else {
                    egui::Grid::new("file_grid")
                    .striped(true)
                    .spacing(egui::vec2(10.0, 8.0))
                    .show(ui, |ui| {
                        // header
                        ui.label("File");
                        ui.label("Kind");
                        ui.label("Size");
                        ui.label("Date");
                        ui.label("★");
                        ui.label("⚑");
                        ui.end_row();

                        // rows
                        for (row_idx, &fid) in filtered_ids.iter().enumerate() {
                            let f = self.index.as_ref().and_then(|idx| idx.files.iter().find(|f| f.id == fid));
                            let Some(f) = f else { continue; };
                            // Clone MediaFile to avoid borrow issues in closure
                            let f_clone = f.clone();
                            let selected = self.selected_ids.contains(&f_clone.id);
                            ui.scope(|ui| {
                                let row_bg = if selected {
                                    egui::Color32::from_gray(60)
                                } else {
                                    egui::Color32::WHITE
                                };
                                // thumbnail
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        if let Some(thumb) = self.thumb_color_image(&f_clone, 96) {
                                            let tex = ctx.load_texture(
                                                format!("thumb_{}", f_clone.id),
                                                    thumb,
                                                    egui::TextureOptions::NEAREST,
                                            );
                                            ui.add(
                                                egui::Image::new((tex.id(), tex.size_vec2()))
                                                .max_width(96.0)
                                                .max_height(96.0),
                                            );
                                        } else {
                                            ui.add_space(64.0);
                                        }
                                    });
                                    ui.vertical(|ui| {
                                        let frame = egui::Frame::group(ui.style())
                                        .fill(row_bg)
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(egui::Margin::same(8.0));
                                        let frame_response = frame.show(ui, |ui| {
                                            ui.label(
                                                f_clone.path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
                                            );
                                            ui.label(f_clone.kind.to_string());
                                            ui.label(format!("{}  {}", f_clone.size, f_clone.modified));
                                        });
                                        // Handle click on the frame area
                                        if frame_response.response.clicked() {
                                            let modifiers = ui.input(|i| i.modifiers);
                                            if modifiers.shift && self.last_clicked_id.is_some() {
                                                // Shift+click: select range
                                                let last_idx = filtered_ids.iter().position(|x| *x == self.last_clicked_id.unwrap()).unwrap_or(row_idx);
                                                let (start, end) = if last_idx <= row_idx { (last_idx, row_idx) } else { (row_idx, last_idx) };
                                                for i in start..=end {
                                                    self.selected_ids.insert(filtered_ids[i]);
                                                }
                                            } else if modifiers.ctrl {
                                                // Ctrl+click: toggle
                                                if selected {
                                                    self.selected_ids.remove(&f_clone.id);
                                                } else {
                                                    self.selected_ids.insert(f_clone.id);
                                                }
                                                self.last_clicked_id = Some(f_clone.id);
                                            } else {
                                                // Single click: select only this
                                                self.selected_ids.clear();
                                                self.selected_ids.insert(f_clone.id);
                                                self.last_clicked_id = Some(f_clone.id);
                                            }
                                        }
                                        // Double-click to open lightbox
                                        if frame_response.response.double_clicked() {
                                            self.preview_id = Some(f_clone.id);
                                        }
                                    });
                                });

                                if selected {
                                    ui.label("✓");
                                }
                                // Rating control (0-5)
                                let mut new_rating = f_clone.rating;
                                ui.add(egui::DragValue::new(&mut new_rating).clamp_range(0..=5).speed(1.0));
                                if new_rating != f_clone.rating {
                                    rating_changes.push((f_clone.id, new_rating));
                                }
                                // Flagged toggle
                                let flag_btn = if f_clone.flagged { "⚑" } else { "☆" };
                                if ui.button(flag_btn).clicked() {
                                    flagged_changes.push((f_clone.id, !f_clone.flagged));
                                }
                                ui.end_row();
                            });
                        }
                    });

                    // ---- projects panel (inside scroll) ----
                    ui.add_space(16.0);
                    egui::CollapsingHeader::new("Projects")
                        .default_open(true)
                        .show(ui, |ui| {
                            let Some(ref idx) = self.index else {
                                ui.label("Index not loaded.");
                                return;
                            };
                            // New project creation
                            ui.horizontal(|ui| {
                                ui.label("New project:");
                                ui.text_edit_singleline(&mut self.new_project_name);
                                if ui.button("Create").clicked() {
                                    create_project = true;
                                    new_project_name_buf = self.new_project_name.trim().to_string();
                                }
                            });
                            ui.add_space(8.0);

                            // Project list
                            if idx.projects.is_empty() {
                                ui.label("No projects yet.");
                            } else {
                                egui::Grid::new("project_grid")
                                    .striped(true)
                                    .spacing(egui::vec2(10.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label("Name");
                                        ui.label("Files");
                                        ui.label("");
                                        ui.end_row();

                                        for proj in &idx.projects {
                                            let selected = self.selected_project_id == Some(proj.id);
                                            let row_bg = if selected {
                                                egui::Color32::from_gray(60)
                                            } else {
                                                egui::Color32::WHITE
                                            };

                                            egui::Frame::group(ui.style())
                                                .fill(row_bg)
                                                .rounding(egui::Rounding::same(4.0))
                                                .inner_margin(egui::Margin::same(6.0))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        if ui
                                                            .button(if selected { "✓" } else { "" })
                                                            .clicked()
                                                        {
                                                            select_project_id = Some(proj.id);
                                                        }
                                                        ui.label(&proj.name);
                                                        ui.label(format!("{}", proj.media_ids.len()));
                                                        if ui.button("✕").clicked() {
                                                            remove_project_id = Some(proj.id);
                                                        }
                                                    });
                                                });
                                            ui.end_row();
                                        }
                                    });

                                ui.add_space(8.0);

                                // Project actions
                                if let Some(proj_id) = self.selected_project_id {
                                    if let Some(proj) = idx.projects.iter().find(|p| p.id == proj_id) {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Selected: {}", proj.name));
                                            if ui.button("Export project…").clicked() {
                                                export_project = true;
                                            }
                                            if ui.button("Clear selection").clicked() {
                                                deselect_project = true;
                                            }
                                        });
                                        ui.label("Destination:");
                                        ui.text_edit_singleline(&mut self.export_dest);
                                        if ui.button("Export").clicked() {
                                            do_export_project = true;
                                        }
                                    }
                                }
                            }
                        });
                }
            });

            // ---- keyboard navigation ----
            if let Some(ref idx) = self.index {
                let filtered_ids: Vec<Uuid> = idx.files
                .iter()
                .filter(|f| {
                    if let Some(t) = &self.filter_tags {
                        let q: Vec<&str> = t
                        .split_whitespace()
                        .filter(|s| !s.is_empty())
                        .collect();
                        if q.is_empty() { return true; }
                        let mut matched = false;
                        for q in &q {
                            let q_lower = q.to_lowercase();
                            if f.path
                                .file_name()
                                .map(|s| s.to_string_lossy().to_lowercase().contains(&q_lower))
                                .unwrap_or(false)
                            {
                                matched = true;
                            }
                            for tag in &f.tags {
                                if tag.to_lowercase().contains(&q_lower) {
                                    matched = true;
                                }
                            }
                        }
                        matched
                    } else {
                        true
                    }
                })
                .map(|f| f.id)
                .collect();

                if !filtered_ids.is_empty() {
                    let mut next_preview_id = None;
                    let mut clear_selection = false;
                    ui.input(|i| {
                        if i.key_pressed(egui::Key::ArrowDown) {
                            // Find currently selected or first item
                            let current_idx = if let Some(last) = self.last_clicked_id {
                                filtered_ids.iter().position(|x| *x == last).unwrap_or(0)
                            } else if let Some(first_sel) = self.selected_ids.iter().next() {
                                filtered_ids.iter().position(|x| *x == *first_sel).unwrap_or(0)
                            } else {
                                0
                            };
                            let next_idx = (current_idx + 1).min(filtered_ids.len() - 1);
                            if i.modifiers.shift {
                                // Extend selection
                                for i in current_idx..=next_idx {
                                    self.selected_ids.insert(filtered_ids[i]);
                                }
                            } else {
                                // Move selection
                                self.selected_ids.clear();
                                self.selected_ids.insert(filtered_ids[next_idx]);
                            }
                            self.last_clicked_id = Some(filtered_ids[next_idx]);
                        }
                        if i.key_pressed(egui::Key::ArrowUp) {
                            let current_idx = if let Some(last) = self.last_clicked_id {
                                filtered_ids.iter().position(|x| *x == last).unwrap_or(0)
                            } else if let Some(first_sel) = self.selected_ids.iter().next() {
                                filtered_ids.iter().position(|x| *x == *first_sel).unwrap_or(0)
                            } else {
                                0
                            };
                            let next_idx = current_idx.saturating_sub(1);
                            if i.modifiers.shift {
                                for i in next_idx..=current_idx {
                                    self.selected_ids.insert(filtered_ids[i]);
                                }
                            } else {
                                self.selected_ids.clear();
                                self.selected_ids.insert(filtered_ids[next_idx]);
                            }
                            self.last_clicked_id = Some(filtered_ids[next_idx]);
                        }
                        if i.key_pressed(egui::Key::Enter) {
                            if let Some(id) = self.last_clicked_id.or_else(|| self.selected_ids.iter().next().copied()) {
                                next_preview_id = Some(id);
                            }
                        }
                        if i.key_pressed(egui::Key::Escape) {
                            if self.preview_id.is_some() {
                                clear_selection = true;
                            } else {
                                self.selected_ids.clear();
                                self.last_clicked_id = None;
                            }
                        }
                        if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
                            // Select all filtered
                            for id in &filtered_ids {
                                self.selected_ids.insert(*id);
                            }
                            self.last_clicked_id = filtered_ids.last().copied();
                        }
                    });
                    if let Some(id) = next_preview_id {
                        self.preview_id = Some(id);
                    }
                    if clear_selection {
                        self.preview_id = None;
                    }
                }
            }

            // ---- keyboard navigation end ----
            // Apply mutations (mutable borrow of self.index)
            if create_project && !new_project_name_buf.is_empty() {
                if let Some(ref mut index) = self.index {
                    index.projects.push(Project {
                        id: Uuid::new_v4(),
                        name: new_project_name_buf,
                        media_ids: Vec::new(),
                    });
                }
                self.new_project_name.clear();
            }
            if let Some(id) = remove_project_id {
                if let Some(ref mut index) = self.index {
                    index.projects.retain(|p| p.id != id);
                }
                if self.selected_project_id == Some(id) {
                    self.selected_project_id = None;
                }
            }
            if select_project_id.is_some() {
                self.selected_project_id = select_project_id;
            }
            if deselect_project {
                self.selected_project_id = None;
            }
            if export_project {
                // Trigger export of selected project via the export menu flow
                self.preview_id = None;
            }
            if do_export_project && !self.export_dest.trim().is_empty() {
                if let Some(ref idx) = self.index {
                    if let Some(proj_id) = self.selected_project_id {
                        if let Some(proj) = idx.projects.iter().find(|p| p.id == proj_id) {
                            let media: Vec<MediaFile> = idx
                                .files
                                .iter()
                                .filter(|f| proj.media_ids.contains(&f.id))
                                .cloned()
                                .collect();
                            match export_files(
                                &media,
                                std::path::Path::new(self.export_dest.trim()),
                            ) {
                                Ok(n) => {
                                    self.status_message = Some(format!(
                                        "Exported {} files to {}",
                                        n,
                                        self.export_dest.trim()
                                    ));
                                }
                                Err(e) => {
                                    self.status_message = Some(format!(
                                        "Project export error: {}",
                                        e
                                    ));
                                }
                            }
                            self.export_dest.clear();
                        }
                    }
                }
            }

            // Apply rating/flagged changes
            if let Some(ref mut index) = self.index {
                for (fid, rating) in &rating_changes {
                    if let Some(f) = index.files.iter_mut().find(|f| f.id == *fid) {
                        f.rating = *rating;
                    }
                }
                for (fid, flagged) in &flagged_changes {
                    if let Some(f) = index.files.iter_mut().find(|f| f.id == *fid) {
                        f.flagged = *flagged;
                    }
                }
            }

            // Auto-save index after any mutation
            let need_save = create_project || remove_project_id.is_some()
                || !rating_changes.is_empty() || !flagged_changes.is_empty();
            if need_save {
                if let Err(e) = self.save_index() {
                    self.status_message = Some(format!("Save error: {}", e));
                } else if create_project || remove_project_id.is_some() {
                    self.status_message = Some("Index saved.".to_string());
                }
            }

            // ---- lightbox (preview) ----
            if let Some(ref idx) = self.index {
                if let Some(id) = self.preview_id {
                    if let Some(f) = idx.files.iter().find(|f| f.id == id) {
                        let f_clone = f.clone();
                        let mut close_lightbox = false;
                        // Lightbox edit intents
                        let mut lb_rating: Option<u8> = None;
                        let mut lb_flagged: Option<bool> = None;
                        let mut lb_tags: Option<String> = None;
                        egui::Window::new("lightbox")
                        .resizable(true)
                        .title_bar(false)
                        .movable(false)
                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                        .show(ctx, |ui| {
                            if let Some(thumb) = self.thumb_color_image(&f_clone, 600) {
                                let tex = ctx.load_texture(format!("lightbox_{}", f_clone.id), thumb, egui::TextureOptions::NEAREST);
                                ui.add(egui::Image::new((tex.id(), tex.size_vec2())).fit_to_exact_size(egui::vec2(600.0, 600.0)));
                            } else {
                                ui.add_space(120.0);
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(600.0, 600.0)),
                                    0.0,
                                    egui::Color32::WHITE,
                                );
                            }
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(8.0);
                            ui.label(format!("File: {}", f_clone.path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default()));
                            ui.label(format!("Kind: {}", f_clone.kind));
                            ui.label(format!("Size: {} bytes", f_clone.size));
                            ui.label(format!("Modified: {}", f_clone.modified));
                            // Rating edit
                            ui.horizontal(|ui| {
                                ui.label("Rating:");
                                let mut r = f_clone.rating;
                                ui.add(egui::DragValue::new(&mut r).clamp_range(0..=5).speed(1.0));
                                if r != f_clone.rating {
                                    lb_rating = Some(r);
                                }
                            });
                            // Flagged toggle
                            ui.horizontal(|ui| {
                                ui.label("Flagged:");
                                let flag_btn = if f_clone.flagged { "⚑" } else { "☆" };
                                if ui.button(flag_btn).clicked() {
                                    lb_flagged = Some(!f_clone.flagged);
                                }
                            });
                            // Tags edit
                            if !f_clone.tags.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("Tags:");
                                    let mut tags = f_clone.tags.join(", ");
                                    ui.text_edit_singleline(&mut tags);
                                    if tags != f_clone.tags.join(", ") {
                                        lb_tags = Some(tags);
                                    }
                                });
                            }
                            ui.add_space(12.0);
                            if ui.button("✕ Close").clicked() {
                                close_lightbox = true;
                            }
                        });

                        // Apply lightbox edits
                        if let Some(ref mut index) = self.index {
                            if let Some(f) = index.files.iter_mut().find(|f| f.id == id) {
                                if let Some(r) = lb_rating { f.rating = r; }
                                if let Some(fl) = lb_flagged { f.flagged = fl; }
                                if let Some(tags_str) = lb_tags {
                                    f.tags = tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                                }
                            }
                        }

                        if close_lightbox {
                            self.preview_id = None;
                        }
                    }
                }
            }
        });
    }
}
