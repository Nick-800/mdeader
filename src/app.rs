use crate::config::Config;
use crate::document::{Document, SearchMatch};
use crate::export::HtmlExporter;
use crate::image_loader::ImageCache;
use crate::renderer::{MarkdownRenderer, RenderEvent};
use crate::syntax::SyntaxHighlighter;
use crate::theme::{ThemeKind, ThemePalette};
use crate::toc::TocView;
use crate::watcher::FileWatcher;
use arboard::Clipboard;
use eframe::egui;
use egui::{
    Align, CentralPanel, Color32, Context, Key, Layout, ScrollArea, SidePanel,
    TextEdit, TopBottomPanel, Ui,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct MdeaderApp {
    pub config: Config,
    pub document: Option<Document>,
    pub current_path: Option<PathBuf>,
    pub watcher: FileWatcher,
    pub highlighter: SyntaxHighlighter,
    pub image_cache: ImageCache,
    pub toc_view: TocView,

    // Search state
    pub search_query: String,
    pub search_open: bool,
    pub search_results: Vec<SearchMatch>,
    pub active_search_idx: usize,

    // Navigation state
    pub active_heading_idx: Option<usize>,
    pub target_heading_idx: Option<usize>,

    // Clipboard and Toast
    pub clipboard: Option<Clipboard>,
    pub toast: Option<(String, Instant)>,
}

impl MdeaderApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_file: Option<PathBuf>,
        initial_theme: Option<String>,
        initial_zoom: Option<f32>,
    ) -> Self {
        let mut config = Config::load();
        if let Some(t) = initial_theme {
            config.theme = t;
        }
        if let Some(z) = initial_zoom {
            config.zoom = z;
        }

        let theme_kind = ThemeKind::from_id(&config.theme);
        cc.egui_ctx.set_visuals(theme_kind.palette().to_visuals(theme_kind.is_dark()));

        let clipboard = Clipboard::new().ok();
        let mut app = Self {
            config,
            document: None,
            current_path: None,
            watcher: FileWatcher::new(),
            highlighter: SyntaxHighlighter::new(),
            image_cache: ImageCache::new(),
            toc_view: TocView::new(),
            search_query: String::new(),
            search_open: false,
            search_results: Vec::new(),
            active_search_idx: 0,
            active_heading_idx: None,
            target_heading_idx: None,
            clipboard,
            toast: None,
        };

        if let Some(path) = initial_file {
            app.open_file(&path);
        } else if let Some(recent) = app.config.recent_files.first().cloned() {
            if recent.exists() {
                app.open_file(&recent);
            }
        }

        app
    }

    pub fn open_file(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let doc = Document::parse(&content);
                self.document = Some(doc);
                self.current_path = Some(path.to_path_buf());
                self.config.add_recent_file(path);
                self.image_cache.clear();
                self.target_heading_idx = None;
                self.active_heading_idx = None;

                if self.config.watch_mode {
                    let _ = self.watcher.watch(path);
                }

                self.show_toast(format!("Opened {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("file")));
            }
            Err(e) => {
                self.show_toast(format!("Error reading file: {}", e));
            }
        }
    }

    pub fn reload_current_file(&mut self) {
        if let Some(ref path) = self.current_path.clone() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let doc = Document::parse(&content);
                self.document = Some(doc);
                if !self.search_query.is_empty() {
                    self.update_search();
                }
                self.show_toast("Reloaded file".to_string());
            }
        }
    }

    pub fn show_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now()));
    }

    pub fn copy_to_clipboard(&mut self, text: &str) {
        if let Some(ref mut cb) = self.clipboard {
            if cb.set_text(text).is_ok() {
                self.show_toast("Copied to clipboard".to_string());
                return;
            }
        }
        self.show_toast("Failed to copy to clipboard".to_string());
    }

    pub fn trigger_open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown Files", &["md", "markdown", "mdown", "mkd"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.open_file(&path);
        }
    }

    pub fn trigger_export_dialog(&mut self) {
        let Some(ref doc) = self.document else {
            self.show_toast("No document open to export".to_string());
            return;
        };

        let default_name = self
            .current_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let default_filename = format!("{}.html", default_name);

        if let Some(dest) = rfd::FileDialog::new()
            .set_file_name(&default_filename)
            .add_filter("HTML Document", &["html", "htm"])
            .save_file()
        {
            let theme_kind = ThemeKind::from_id(&self.config.theme);
            let palette = theme_kind.palette();
            let title = default_name;

            match HtmlExporter::save_to_file(&doc.raw, title, &palette, &dest) {
                Ok(_) => self.show_toast(format!("Exported to {}", dest.display())),
                Err(e) => self.show_toast(format!("Export failed: {}", e)),
            }
        }
    }

    fn update_search(&mut self) {
        if let Some(ref doc) = self.document {
            self.search_results = doc.search(&self.search_query);
            self.active_search_idx = 0;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        let input = ctx.input(|i| i.clone());

        // Ctrl+O: Open
        if input.modifiers.command && input.key_pressed(Key::O) {
            self.trigger_open_file_dialog();
        }

        // Ctrl+R: Reload
        if input.modifiers.command && input.key_pressed(Key::R) {
            self.reload_current_file();
        }

        // Ctrl+F: Find
        if input.modifiers.command && input.key_pressed(Key::F) {
            self.search_open = !self.search_open;
        }

        // Escape: Close Find
        if input.key_pressed(Key::Escape) && self.search_open {
            self.search_open = false;
        }

        // Ctrl+B: Toggle TOC
        if input.modifiers.command && input.key_pressed(Key::B) {
            self.config.show_toc = !self.config.show_toc;
            let _ = self.config.save();
        }

        // Ctrl+E: Export
        if input.modifiers.command && input.key_pressed(Key::E) {
            self.trigger_export_dialog();
        }

        // Zoom: Ctrl+= or Ctrl++
        if input.modifiers.command && (input.key_pressed(Key::Equals) || input.key_pressed(Key::Plus)) {
            self.config.zoom = (self.config.zoom + 0.1).min(2.5);
            let _ = self.config.save();
        }

        // Zoom: Ctrl+-
        if input.modifiers.command && input.key_pressed(Key::Minus) {
            self.config.zoom = (self.config.zoom - 0.1).max(0.6);
            let _ = self.config.save();
        }

        // Zoom: Ctrl+0 (Reset)
        if input.modifiers.command && input.key_pressed(Key::Num0) {
            self.config.zoom = 1.0;
            let _ = self.config.save();
        }

        // Drag & Drop
        for file in &input.raw.dropped_files {
            if let Some(ref path) = file.path {
                self.open_file(path);
                break;
            }
        }
    }

    fn render_top_bar(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        ui.horizontal(|ui| {
            if ui.button("📂 Open (Ctrl+O)").clicked() {
                self.trigger_open_file_dialog();
            }

            ui.menu_button("Recent", |ui| {
                if self.config.recent_files.is_empty() {
                    ui.label("No recent files");
                } else {
                    let recents = self.config.recent_files.clone();
                    for path in recents {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                        if ui.button(name).on_hover_text(path.display().to_string()).clicked() {
                            self.open_file(&path);
                            ui.close_menu();
                        }
                    }
                }
            });

            if ui.button("🔄 Reload").clicked() {
                self.reload_current_file();
            }

            ui.separator();

            let toc_btn_text = if self.config.show_toc { "📖 Hide TOC" } else { "📖 TOC" };
            if ui.selectable_label(self.config.show_toc, toc_btn_text).clicked() {
                self.config.show_toc = !self.config.show_toc;
                let _ = self.config.save();
            }

            let search_btn_text = if self.search_open { "🔍 Search" } else { "🔍 Find" };
            if ui.selectable_label(self.search_open, search_btn_text).clicked() {
                self.search_open = !self.search_open;
            }

            ui.separator();

            // Theme selection dropdown
            let current_theme = ThemeKind::from_id(&self.config.theme);
            let mut selected_theme = current_theme;

            egui::ComboBox::from_id_source("theme_selector")
                .selected_text(current_theme.name())
                .show_ui(ui, |ui| {
                    for t in ThemeKind::ALL {
                        ui.selectable_value(&mut selected_theme, *t, t.name());
                    }
                });

            if selected_theme != current_theme {
                self.config.theme = selected_theme.id().to_string();
                let _ = self.config.save();
                ui.ctx().set_visuals(selected_theme.palette().to_visuals(selected_theme.is_dark()));
            }

            ui.separator();

            // Zoom controls
            if ui.small_button("−").on_hover_text("Zoom out (Ctrl+-)").clicked() {
                self.config.zoom = (self.config.zoom - 0.1).max(0.6);
                let _ = self.config.save();
            }

            let zoom_pct = format!("{:.0}%", self.config.zoom * 100.0);
            if ui.small_button(zoom_pct).on_hover_text("Reset zoom (Ctrl+0)").clicked() {
                self.config.zoom = 1.0;
                let _ = self.config.save();
            }

            if ui.small_button("+").on_hover_text("Zoom in (Ctrl++)").clicked() {
                self.config.zoom = (self.config.zoom + 0.1).min(2.5);
                let _ = self.config.save();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("📤 Export HTML").clicked() {
                    self.trigger_export_dialog();
                }

                if ui.button("📋 Copy Raw").on_hover_text("Copy entire Markdown source").clicked() {
                    if let Some(ref doc) = self.document {
                        let raw = doc.raw.clone();
                        self.copy_to_clipboard(&raw);
                    }
                }
            });
        });

        // Search Bar (if open)
        if self.search_open {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Search:");
                let text_edit = TextEdit::singleline(&mut self.search_query)
                    .hint_text("Type to search in document...")
                    .desired_width(260.0);

                let response = ui.add(text_edit);
                if response.changed() {
                    self.update_search();
                }

                let match_count = self.search_results.len();
                let match_text = if match_count == 0 {
                    if self.search_query.is_empty() {
                        "".to_string()
                    } else {
                        "No matches".to_string()
                    }
                } else {
                    format!("{}/{} matches", self.active_search_idx + 1, match_count)
                };

                ui.colored_label(palette.muted, match_text);

                if match_count > 0 {
                    if ui.small_button("▲").clicked() {
                        if self.active_search_idx == 0 {
                            self.active_search_idx = match_count - 1;
                        } else {
                            self.active_search_idx -= 1;
                        }
                    }
                    if ui.small_button("▼").clicked() {
                        self.active_search_idx = (self.active_search_idx + 1) % match_count;
                    }
                }

                if ui.small_button("✕").clicked() {
                    self.search_open = false;
                    self.search_query.clear();
                    self.search_results.clear();
                }
            });
        }
    }

    fn render_status_bar(&self, ui: &mut Ui, palette: &ThemePalette) {
        ui.horizontal(|ui| {
            // File path
            if let Some(ref path) = self.current_path {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                ui.strong(name);
                ui.colored_label(palette.muted, format!("({})", path.display()));
            } else {
                ui.colored_label(palette.muted, "No document opened");
            }

            ui.separator();

            // Stats
            if let Some(ref doc) = self.document {
                let s = &doc.stats;
                let stat_text = format!(
                    "{} words | {} chars | {} lines | ~{} min read",
                    s.word_count, s.char_count, s.line_count, s.reading_time_mins
                );
                ui.colored_label(palette.muted, stat_text);
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Toast
                if let Some((ref msg, ref instant)) = self.toast {
                    if instant.elapsed().as_secs() < 3 {
                        ui.colored_label(palette.accent, format!("✓ {}", msg));
                    }
                }

                // Watch indicator
                if self.config.watch_mode && self.watcher.is_watching() {
                    ui.colored_label(Color32::from_rgb(46, 160, 67), "● Live Watch");
                } else {
                    ui.colored_label(palette.muted, "○ Static");
                }
            });
        });
    }
}

impl eframe::App for MdeaderApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);

        // Check file watcher events
        if self.config.watch_mode && self.watcher.has_changes() {
            self.reload_current_file();
        }

        let theme_kind = ThemeKind::from_id(&self.config.theme);
        let palette = theme_kind.palette();

        // Top Panel
        TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            self.render_top_bar(ui, &palette);
        });

        // Bottom Panel
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui, &palette);
        });

        // Left Outline Panel (TOC)
        if self.config.show_toc {
            SidePanel::left("toc_sidebar")
                .default_width(self.config.toc_width)
                .min_width(180.0)
                .max_width(450.0)
                .show(ctx, |ui| {
                    if let Some(ref doc) = self.document {
                        if let Some(clicked) = self.toc_view.ui(
                            ui,
                            &doc.headings,
                            &palette,
                            self.active_heading_idx,
                        ) {
                            self.target_heading_idx = Some(clicked);
                            self.active_heading_idx = Some(clicked);
                        }
                    } else {
                        ui.colored_label(palette.muted, "No document opened");
                    }
                });
        }

        // Central Content Area
        CentralPanel::default().show(ctx, |ui| {
            if let Some(ref doc) = self.document {
                let base_dir = self.current_path.as_deref().and_then(|p| p.parent());
                let effective_font_size = self.config.font_size * self.config.zoom;
                let effective_line_spacing = self.config.line_spacing;

                let mut renderer = MarkdownRenderer::new(
                    base_dir,
                    &palette,
                    effective_font_size,
                    effective_line_spacing,
                    &self.search_query,
                    self.target_heading_idx,
                );

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(12.0);
                        renderer.render(ui, &doc.nodes, &self.highlighter, &mut self.image_cache);
                        ui.add_space(32.0);
                    });

                // Clear target heading once rendered
                self.target_heading_idx = None;

                // Process renderer events
                for event in renderer.events {
                    match event {
                        RenderEvent::OpenLink(url) => {
                            let _ = open::that(url);
                        }
                        RenderEvent::OpenFilePath(path) => {
                            self.open_file(&path);
                        }
                        RenderEvent::CopyToClipboard(text) => {
                            self.copy_to_clipboard(&text);
                        }
                    }
                }
            } else {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("mdeader");
                        ui.add_space(8.0);
                        ui.colored_label(palette.muted, "A standalone, fast Markdown reader in Rust");
                        ui.add_space(16.0);

                        if ui.button("📂 Open Markdown File (Ctrl+O)").clicked() {
                            self.trigger_open_file_dialog();
                        }

                        ui.add_space(8.0);
                        ui.colored_label(palette.muted, "Or drag & drop a .md file here");
                    });
                });
            }
        });
    }
}
