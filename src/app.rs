use crate::ai::{context::DocumentContext, view::AiView, AiManager};
use crate::config::Config;
use crate::document::{Document, SearchMatch};
use crate::export::HtmlExporter;
use crate::image_loader::ImageCache;
use crate::ipc::{IpcCommand, IpcServer};
use crate::renderer::{MarkdownRenderer, RenderEvent};
use crate::syntax::SyntaxHighlighter;
use crate::theme::{ThemeKind, ThemePalette};
use crate::theme_loader::{CustomTheme, ThemeLoader};
use crate::toc::TocView;
use crate::vim::{VimAction, VimController};
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

    // AI Assistant state
    pub ai: AiManager,

    // Remote IPC Server
    pub ipc_server: Option<IpcServer>,

    // Vim Navigation
    pub vim: VimController,

    // Custom Themes
    pub custom_themes: Vec<CustomTheme>,

    // Specialized Reading Modes
    pub zen_mode: bool,
    pub slide_mode: bool,
    pub current_slide: usize,

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

    // Dynamic UI interaction
    pub scroll_delta_y: f32,
    pub focus_search_input: bool,
}

impl MdeaderApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_file: Option<PathBuf>,
        initial_theme: Option<String>,
        initial_zoom: Option<f32>,
        initial_vim: bool,
    ) -> Self {
        let mut config = Config::load();
        if let Some(t) = initial_theme {
            config.theme = t;
        }
        if let Some(z) = initial_zoom {
            config.zoom = z;
        }
        if initial_vim {
            config.vim_mode = true;
        }

        let custom_themes = ThemeLoader::load_all();

        let (initial_palette, is_dark) = if let Some(custom) = custom_themes.iter().find(|t| t.id == config.theme) {
            (custom.palette.clone(), custom.is_dark)
        } else {
            let theme_kind = ThemeKind::from_id(&config.theme);
            (theme_kind.palette(), theme_kind.is_dark())
        };
        cc.egui_ctx.set_visuals(initial_palette.to_visuals(is_dark));

        let clipboard = Clipboard::new().ok();
        let ipc_server = if config.ipc_enabled {
            IpcServer::start(config.ipc_port, cc.egui_ctx.clone())
        } else {
            None
        };

        let vim = VimController::new(config.vim_mode);

        let mut app = Self {
            config,
            document: None,
            current_path: None,
            watcher: FileWatcher::new(),
            highlighter: SyntaxHighlighter::new(),
            image_cache: ImageCache::new(),
            toc_view: TocView::new(),
            ai: AiManager::new(),
            ipc_server,
            vim,
            custom_themes,
            zen_mode: false,
            slide_mode: false,
            current_slide: 0,
            search_query: String::new(),
            search_open: false,
            search_results: Vec::new(),
            active_search_idx: 0,
            active_heading_idx: None,
            target_heading_idx: None,
            clipboard,
            toast: None,
            scroll_delta_y: 0.0,
            focus_search_input: false,
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

    pub fn active_palette(&self) -> (ThemePalette, bool) {
        if let Some(custom) = self.custom_themes.iter().find(|t| t.id == self.config.theme) {
            (custom.palette.clone(), custom.is_dark)
        } else {
            let theme_kind = ThemeKind::from_id(&self.config.theme);
            (theme_kind.palette(), theme_kind.is_dark())
        }
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

    pub fn jump_to_heading(&mut self, title: &str) {
        if let Some(ref doc) = self.document {
            let lower = title.to_lowercase();
            if let Some((idx, _)) = doc.headings.iter().enumerate().find(|(_, h)| h.title.to_lowercase().contains(&lower)) {
                self.target_heading_idx = Some(idx);
                self.active_heading_idx = Some(idx);
                let heading_name = doc.headings[idx].title.clone();
                self.show_toast(format!("Jumped to {}", heading_name));
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
        // Vim mode keyboard handling
        if let Some(action) = self.vim.handle_input(ctx) {
            match action {
                VimAction::ScrollDownLine => {
                    self.scroll_delta_y -= 60.0;
                }
                VimAction::ScrollUpLine => {
                    self.scroll_delta_y += 60.0;
                }
                VimAction::ScrollDownHalfPage => {
                    self.scroll_delta_y -= 350.0;
                }
                VimAction::ScrollUpHalfPage => {
                    self.scroll_delta_y += 350.0;
                }
                VimAction::ScrollTop => {
                    self.scroll_delta_y += 100_000.0;
                    self.target_heading_idx = Some(0);
                    self.active_heading_idx = Some(0);
                }
                VimAction::ScrollBottom => {
                    self.scroll_delta_y -= 100_000.0;
                    if let Some(ref doc) = self.document {
                        if !doc.headings.is_empty() {
                            let last = doc.headings.len() - 1;
                            self.target_heading_idx = Some(last);
                            self.active_heading_idx = Some(last);
                        }
                    }
                }
                VimAction::FocusSearch => {
                    self.search_open = true;
                    self.focus_search_input = true;
                }
                VimAction::SearchNext => {
                    let count = self.search_results.len();
                    if count > 0 {
                        self.active_search_idx = (self.active_search_idx + 1) % count;
                    }
                }
                VimAction::SearchPrev => {
                    let count = self.search_results.len();
                    if count > 0 {
                        if self.active_search_idx == 0 {
                            self.active_search_idx = count - 1;
                        } else {
                            self.active_search_idx -= 1;
                        }
                    }
                }
                VimAction::ToggleToc => {
                    self.config.show_toc = !self.config.show_toc;
                    let _ = self.config.save();
                }
                VimAction::ToggleAi => {
                    self.config.show_ai = !self.config.show_ai;
                    let _ = self.config.save();
                }
                VimAction::ToggleZen => {
                    self.zen_mode = !self.zen_mode;
                }
                VimAction::ToggleSlideMode => {
                    self.slide_mode = !self.slide_mode;
                    self.current_slide = 0;
                }
                VimAction::Reload => {
                    self.reload_current_file();
                }
                VimAction::OpenFile => {
                    self.trigger_open_file_dialog();
                }
                VimAction::CopyRaw => {
                    if let Some(ref doc) = self.document {
                        let raw = doc.raw.clone();
                        self.copy_to_clipboard(&raw);
                    }
                }
                VimAction::QuitOrEscape => {
                    if self.slide_mode {
                        self.slide_mode = false;
                    } else if self.zen_mode {
                        self.zen_mode = false;
                    } else if self.search_open {
                        self.search_open = false;
                    }
                }
            }
        }

        let input = ctx.input(|i| i.clone());
        let cmd = input.modifiers.command || input.modifiers.ctrl;

        // F11: Toggle Zen Mode
        if input.key_pressed(Key::F11) {
            self.zen_mode = !self.zen_mode;
        }

        // F5: Toggle Presentation Slide Mode
        if input.key_pressed(Key::F5) {
            self.slide_mode = !self.slide_mode;
            self.current_slide = 0;
        }

        // Slide Mode keyboard controls
        if self.slide_mode {
            let total_slides = self.document.as_ref().map(|d| d.get_slides().len()).unwrap_or(0);
            if total_slides > 0 {
                if input.key_pressed(Key::ArrowRight)
                    || input.key_pressed(Key::Space)
                    || input.key_pressed(Key::PageDown)
                    || input.key_pressed(Key::L)
                    || input.key_pressed(Key::ArrowDown)
                {
                    self.current_slide = (self.current_slide + 1).min(total_slides - 1);
                }
                if input.key_pressed(Key::ArrowLeft)
                    || input.key_pressed(Key::Backspace)
                    || input.key_pressed(Key::PageUp)
                    || input.key_pressed(Key::H)
                    || input.key_pressed(Key::ArrowUp)
                {
                    self.current_slide = self.current_slide.saturating_sub(1);
                }
            }
        }

        // Escape: Close Find, Exit Zen Mode, or Exit Slide Mode
        if input.key_pressed(Key::Escape) {
            if self.slide_mode {
                self.slide_mode = false;
            } else if self.zen_mode {
                self.zen_mode = false;
            } else if self.search_open {
                self.search_open = false;
            }
        }

        // Home: Jump to top of document
        if input.key_pressed(Key::Home) {
            self.scroll_delta_y += 100_000.0;
            self.target_heading_idx = Some(0);
            self.active_heading_idx = Some(0);
        }

        // End: Jump to bottom of document
        if input.key_pressed(Key::End) {
            self.scroll_delta_y -= 100_000.0;
            if let Some(ref doc) = self.document {
                if !doc.headings.is_empty() {
                    let last = doc.headings.len() - 1;
                    self.target_heading_idx = Some(last);
                    self.active_heading_idx = Some(last);
                }
            }
        }

        // Ctrl+O: Open
        if cmd && input.key_pressed(Key::O) {
            self.trigger_open_file_dialog();
        }

        // Ctrl+R: Reload
        if cmd && input.key_pressed(Key::R) {
            self.reload_current_file();
        }

        // Ctrl+F: Find
        if cmd && input.key_pressed(Key::F) {
            self.search_open = !self.search_open;
            if self.search_open {
                self.focus_search_input = true;
            }
        }

        // Ctrl+B: Toggle TOC
        if cmd && input.key_pressed(Key::B) {
            self.config.show_toc = !self.config.show_toc;
            let _ = self.config.save();
        }

        // Ctrl+E: Export
        if cmd && input.key_pressed(Key::E) {
            self.trigger_export_dialog();
        }

        // Zoom: Ctrl+= or Ctrl++ (including Shift+=)
        if cmd && (input.key_pressed(Key::Equals) || input.key_pressed(Key::Plus)) {
            self.config.zoom = (self.config.zoom + 0.1).min(2.5);
            let _ = self.config.save();
        }

        // Zoom: Ctrl+-
        if cmd && input.key_pressed(Key::Minus) {
            self.config.zoom = (self.config.zoom - 0.1).max(0.6);
            let _ = self.config.save();
        }

        // Zoom: Ctrl+0 (Reset)
        if cmd && input.key_pressed(Key::Num0) {
            self.config.zoom = 1.0;
            let _ = self.config.save();
        }

        // Ctrl+Shift+A or Ctrl+I: Toggle AI Drawer
        if (cmd && input.modifiers.shift && input.key_pressed(Key::A))
            || (cmd && input.key_pressed(Key::I))
        {
            self.config.show_ai = !self.config.show_ai;
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
            if ui.button("Open (Ctrl+O)").clicked() {
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

            if ui.button("Reload (Ctrl+R)").clicked() {
                self.reload_current_file();
            }

            ui.separator();

            let toc_btn_text = if self.config.show_toc { "Hide TOC (Ctrl+B)" } else { "TOC (Ctrl+B)" };
            if ui.selectable_label(self.config.show_toc, toc_btn_text).clicked() {
                self.config.show_toc = !self.config.show_toc;
                let _ = self.config.save();
            }

            let search_btn_text = if self.search_open { "Search" } else { "Find (Ctrl+F)" };
            if ui.selectable_label(self.search_open, search_btn_text).clicked() {
                self.search_open = !self.search_open;
                if self.search_open {
                    self.focus_search_input = true;
                }
            }

            let ai_btn_text = if self.config.show_ai { "Hide AI" } else { "AI Assistant" };
            if ui.selectable_label(self.config.show_ai, ai_btn_text).on_hover_text("Toggle AI Assistant (Ctrl+Shift+A)").clicked() {
                self.config.show_ai = !self.config.show_ai;
                let _ = self.config.save();
            }

            ui.separator();

            // Theme selector (built-in + custom)
            let current_theme_name = if let Some(custom) = self.custom_themes.iter().find(|t| t.id == self.config.theme) {
                custom.name.clone()
            } else {
                ThemeKind::from_id(&self.config.theme).name().to_string()
            };

            egui::ComboBox::from_id_source("theme_selector")
                .selected_text(current_theme_name)
                .show_ui(ui, |ui| {
                    ui.label("Built-in Themes:");
                    for t in ThemeKind::ALL {
                        if ui.selectable_label(self.config.theme == t.id(), t.name()).clicked() {
                            self.config.theme = t.id().to_string();
                            let _ = self.config.save();
                            ui.ctx().set_visuals(t.palette().to_visuals(t.is_dark()));
                        }
                    }

                    if !self.custom_themes.is_empty() {
                        ui.separator();
                        ui.label("Custom Themes:");
                        let custom_list = self.custom_themes.clone();
                        for custom in custom_list {
                            if ui.selectable_label(self.config.theme == custom.id, &custom.name).clicked() {
                                self.config.theme = custom.id.clone();
                                let _ = self.config.save();
                                ui.ctx().set_visuals(custom.palette.to_visuals(custom.is_dark));
                            }
                        }
                    }
                });

            ui.separator();

            // Niche Mode controls
            let vim_btn_text = if self.config.vim_mode { "Vim: ON" } else { "Vim" };
            if ui.selectable_label(self.config.vim_mode, vim_btn_text).on_hover_text("Toggle Vim modal navigation").clicked() {
                self.config.vim_mode = !self.config.vim_mode;
                self.vim.enabled = self.config.vim_mode;
                let _ = self.config.save();
            }

            if ui.button("Zen (F11)").on_hover_text("Distraction-free reading canvas").clicked() {
                self.zen_mode = true;
            }

            if ui.button("Slides (F5)").on_hover_text("Presentation slide deck mode").clicked() {
                self.slide_mode = true;
                self.current_slide = 0;
            }

            ui.separator();

            // Zoom controls
            if ui.small_button("-").on_hover_text("Zoom out (Ctrl+-)").clicked() {
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
                if ui.button("Export HTML (Ctrl+E)").clicked() {
                    self.trigger_export_dialog();
                }

                if ui.button("Copy Raw").on_hover_text("Copy entire Markdown source").clicked() {
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
                if self.focus_search_input {
                    response.request_focus();
                    self.focus_search_input = false;
                }
                if response.changed() {
                    self.update_search();
                }

                let match_count = self.search_results.len();

                // Enter and Shift+Enter cycling
                if response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter)) && match_count > 0 {
                    let shift = ui.input(|i| i.modifiers.shift);
                    if shift {
                        if self.active_search_idx == 0 {
                            self.active_search_idx = match_count - 1;
                        } else {
                            self.active_search_idx -= 1;
                        }
                    } else {
                        self.active_search_idx = (self.active_search_idx + 1) % match_count;
                    }
                }

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
                    if ui.small_button("<").on_hover_text("Previous match (Shift+Enter)").clicked() {
                        if self.active_search_idx == 0 {
                            self.active_search_idx = match_count - 1;
                        } else {
                            self.active_search_idx -= 1;
                        }
                    }
                    if ui.small_button(">").on_hover_text("Next match (Enter)").clicked() {
                        self.active_search_idx = (self.active_search_idx + 1) % match_count;
                    }
                }

                if ui.small_button("x").on_hover_text("Close (Esc)").clicked() {
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
                // Vim mode badge
                if self.config.vim_mode {
                    ui.colored_label(palette.accent, self.vim.status_badge());
                }

                // IPC indicator
                if self.config.ipc_enabled && self.ipc_server.is_some() {
                    ui.colored_label(palette.muted, format!("[IPC :{}]", self.config.ipc_port));
                }

                // Toast
                if let Some((ref msg, ref instant)) = self.toast {
                    if instant.elapsed().as_secs() < 3 {
                        ui.colored_label(palette.accent, format!("[OK] {}", msg));
                    }
                }

                // Watch indicator
                if self.config.watch_mode && self.watcher.is_watching() {
                    ui.colored_label(Color32::from_rgb(46, 160, 67), "[Live Watch]");
                } else {
                    ui.colored_label(palette.muted, "[Static]");
                }
            });
        });
    }
}

impl eframe::App for MdeaderApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);

        // Update AI state
        self.ai.update(ctx);

        // Process incoming IPC commands
        let mut pending_commands = Vec::new();
        if let Some(ref server) = self.ipc_server {
            while let Some(cmd) = server.try_recv() {
                pending_commands.push(cmd);
            }
        }

        for cmd in pending_commands {
            match cmd {
                IpcCommand::OpenFile(path) => {
                    self.open_file(&path);
                }
                IpcCommand::JumpToHeading(heading) => {
                    self.jump_to_heading(&heading);
                }
                IpcCommand::Reload => {
                    self.reload_current_file();
                }
                IpcCommand::AskAi(prompt) => {
                    self.config.show_ai = true;
                    let doc_context = DocumentContext::extract_document_context(
                        self.document.as_ref(),
                        self.target_heading_idx.or(self.active_heading_idx),
                    );
                    let system_prompt = DocumentContext::build_system_prompt(
                        self.current_path.as_deref(),
                        self.document.as_ref(),
                    );
                    let full_prompt = format!("{}\n\nUser Question:\n{}", doc_context, prompt);
                    self.ai.send_prompt(&self.config.ai, system_prompt, full_prompt, ctx.clone());
                }
                IpcCommand::Ping => {}
            }
        }

        // Check file watcher events
        if self.config.watch_mode && self.watcher.has_changes() {
            self.reload_current_file();
        }

        let (palette, _is_dark) = self.active_palette();

        // 1. Presentation Slide Deck Mode
        if self.slide_mode {
            CentralPanel::default().show(ctx, |ui| {
                let Some(ref doc) = self.document else {
                    ui.centered_and_justified(|ui| {
                        ui.label("No document open to present");
                    });
                    return;
                };

                let slides = doc.get_slides();
                let total_slides = slides.len();
                if total_slides == 0 {
                    ui.centered_and_justified(|ui| {
                        ui.label("Document is empty");
                    });
                    return;
                }

                if self.current_slide >= total_slides {
                    self.current_slide = total_slides.saturating_sub(1);
                }

                // Slide Header Controls
                ui.horizontal(|ui| {
                    ui.strong(format!("Slide {} of {}", self.current_slide + 1, total_slides));
                    if ui.small_button("< Prev (Left)").clicked() {
                        self.current_slide = self.current_slide.saturating_sub(1);
                    }
                    if ui.small_button("Next > (Right)").clicked() {
                        self.current_slide = (self.current_slide + 1).min(total_slides - 1);
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Exit Slides (Esc / F5)").clicked() {
                            self.slide_mode = false;
                        }
                    });
                });

                ui.separator();
                ui.add_space(24.0);

                let slide_nodes = &slides[self.current_slide];
                let base_dir = self.current_path.as_deref().and_then(|p| p.parent());
                let presentation_font_size = self.config.font_size * 1.35 * self.config.zoom;
                let presentation_line_spacing = self.config.line_spacing * 1.15;

                let mut renderer = MarkdownRenderer::new(
                    base_dir,
                    &palette,
                    presentation_font_size,
                    presentation_line_spacing,
                    "",
                    None,
                );

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.set_max_width(self.config.max_content_width * 1.15);
                            renderer.render(ui, slide_nodes, &self.highlighter, &mut self.image_cache);
                            ui.add_space(40.0);
                        });
                    });
            });
            return;
        }

        // 2. Zen Focus Reading Mode
        if self.zen_mode {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(palette.muted, "[Zen Mode]");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Exit Zen (Esc / F11)").clicked() {
                            self.zen_mode = false;
                        }
                    });
                });

                let mut zen_events = Vec::new();
                let mut current_doc_raw = None;

                if let Some(ref doc) = self.document {
                    current_doc_raw = Some(doc.raw.clone());
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
                            if self.scroll_delta_y != 0.0 {
                                ui.scroll_with_delta(egui::vec2(0.0, self.scroll_delta_y));
                                self.scroll_delta_y = 0.0;
                            }
                            ui.vertical_centered(|ui| {
                                ui.set_max_width(self.config.max_content_width);
                                ui.add_space(20.0);
                                renderer.render(ui, &doc.nodes, &self.highlighter, &mut self.image_cache);
                                ui.add_space(40.0);
                            });
                        });

                    self.target_heading_idx = None;
                    zen_events = renderer.events;
                }

                for event in zen_events {
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
                        RenderEvent::ExplainCode { lang, code } => {
                            self.zen_mode = false;
                            self.config.show_ai = true;
                            let system_prompt = DocumentContext::build_system_prompt(
                                self.current_path.as_deref(),
                                self.document.as_ref(),
                            );
                            self.ai.explain_code(&self.config.ai, system_prompt, &lang, &code, ctx.clone());
                        }
                        RenderEvent::ToggleTask { task_index } => {
                            if let Some(ref raw) = current_doc_raw {
                                if let Some(updated_raw) = Document::toggle_task(raw, task_index) {
                                    if let Some(ref path) = self.current_path {
                                        let _ = std::fs::write(path, &updated_raw);
                                    }
                                    self.document = Some(Document::parse(&updated_raw));
                                    self.show_toast("Task toggled".to_string());
                                }
                            }
                        }
                    }
                }
            });
            return;
        }

        // 3. Standard Reading Environment
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

        // Right AI Assistant Panel
        if self.config.show_ai {
            SidePanel::right("ai_drawer")
                .default_width(self.config.ai_width)
                .min_width(260.0)
                .max_width(600.0)
                .show(ctx, |ui| {
                    AiView::render(
                        ui,
                        &mut self.ai,
                        &mut self.config,
                        self.current_path.as_deref(),
                        self.document.as_ref(),
                        self.target_heading_idx.or(self.active_heading_idx),
                        &palette,
                    );
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
                        if self.scroll_delta_y != 0.0 {
                            ui.scroll_with_delta(egui::vec2(0.0, self.scroll_delta_y));
                            self.scroll_delta_y = 0.0;
                        }
                        ui.add_space(12.0);
                        ui.vertical_centered(|ui| {
                            ui.set_max_width(self.config.max_content_width);
                            renderer.render(ui, &doc.nodes, &self.highlighter, &mut self.image_cache);
                        });
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
                        RenderEvent::ExplainCode { lang, code } => {
                            self.config.show_ai = true;
                            let system_prompt = DocumentContext::build_system_prompt(
                                self.current_path.as_deref(),
                                self.document.as_ref(),
                            );
                            self.ai.explain_code(&self.config.ai, system_prompt, &lang, &code, ctx.clone());
                        }
                        RenderEvent::ToggleTask { task_index } => {
                            if let Some(ref doc) = self.document {
                                if let Some(updated_raw) = Document::toggle_task(&doc.raw, task_index) {
                                    if let Some(ref path) = self.current_path {
                                        if let Err(e) = std::fs::write(path, &updated_raw) {
                                            self.show_toast(format!("Failed to save task update: {}", e));
                                        } else {
                                            self.document = Some(Document::parse(&updated_raw));
                                            self.show_toast("Task toggled and saved".to_string());
                                        }
                                    } else {
                                        self.document = Some(Document::parse(&updated_raw));
                                        self.show_toast("Task toggled".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(logo) = self.image_cache.get_embedded_logo(ui.ctx()) {
                            ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(128.0, 128.0)));
                            ui.add_space(12.0);
                        }

                        ui.heading("mdeader");
                        ui.add_space(8.0);
                        ui.colored_label(palette.muted, "A standalone, fast Markdown reader in Rust");
                        ui.add_space(16.0);

                        if ui.button("Open Markdown File (Ctrl+O)").clicked() {
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
