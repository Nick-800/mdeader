use crate::document::{AlertKind, Alignment, DocNode, InlineSpan, ListItem, TableRow};
use crate::image_loader::ImageCache;
use crate::syntax::SyntaxHighlighter;
use crate::theme::ThemePalette;
use egui::text::LayoutJob;
use egui::{
    vec2, Align, Color32, FontId, Frame, Margin, Pos2, Rect, Response, Rounding, Stroke, TextFormat, Ui,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum RenderEvent {
    OpenLink(String),
    OpenFilePath(PathBuf),
    CopyToClipboard(String),
    ExplainCode { lang: String, code: String },
}

pub struct MarkdownRenderer<'a> {
    pub base_dir: Option<&'a Path>,
    pub palette: &'a ThemePalette,
    pub font_size: f32,
    pub line_spacing: f32,
    pub search_query: &'a str,
    pub target_heading_index: Option<usize>,
    pub events: Vec<RenderEvent>,
}

impl<'a> MarkdownRenderer<'a> {
    pub fn new(
        base_dir: Option<&'a Path>,
        palette: &'a ThemePalette,
        font_size: f32,
        line_spacing: f32,
        search_query: &'a str,
        target_heading_index: Option<usize>,
    ) -> Self {
        Self {
            base_dir,
            palette,
            font_size,
            line_spacing,
            search_query,
            target_heading_index,
            events: Vec::new(),
        }
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        nodes: &[DocNode],
        highlighter: &SyntaxHighlighter,
        images: &mut ImageCache,
    ) {
        for node in nodes {
            self.render_node(ui, node, highlighter, images);
            ui.add_space(8.0 * self.line_spacing);
        }
    }

    fn render_node(
        &mut self,
        ui: &mut Ui,
        node: &DocNode,
        highlighter: &SyntaxHighlighter,
        images: &mut ImageCache,
    ) {
        match node {
            DocNode::Heading {
                level,
                spans,
                heading_index,
                ..
            } => {
                let scale = match level {
                    1 => 1.85,
                    2 => 1.5,
                    3 => 1.28,
                    4 => 1.12,
                    5 => 1.0,
                    _ => 0.9,
                };
                let h_size = self.font_size * scale;
                let lvl_idx = (level.saturating_sub(1) as usize).min(5);
                let heading_color = self.palette.heading_colors[lvl_idx];

                ui.add_space(10.0);

                let id = ui.make_persistent_id(format!("heading_{}", heading_index));
                let mut resp: Option<Response> = None;

                ui.push_id(id, |ui| {
                    let mut job = LayoutJob::default();
                    self.append_spans_to_job(
                        &mut job,
                        spans,
                        h_size,
                        heading_color,
                        true,
                        false,
                        false,
                    );
                    let label_resp = ui.label(job);
                    resp = Some(label_resp);

                    if *level <= 2 {
                        ui.add_space(3.0);
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(ui.available_width(), 1.5),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            Rounding::ZERO,
                            self.palette.code_border,
                        );
                        ui.add_space(4.0);
                    }
                });

                if self.target_heading_index == Some(*heading_index) {
                    if let Some(r) = resp {
                        r.scroll_to_me(Some(Align::TOP));
                    }
                }
            }

            DocNode::Paragraph(spans) => {
                if spans.len() == 1 {
                    if let InlineSpan::Image { url, title, alt } = &spans[0] {
                        self.render_image_block(ui, url, title, alt, images);
                        return;
                    }
                }

                let mut job = LayoutJob::default();
                job.wrap.max_width = ui.available_width();
                self.append_spans_to_job(
                    &mut job,
                    spans,
                    self.font_size,
                    self.palette.text,
                    false,
                    false,
                    false,
                );
                let _ = ui.label(job);

                // Handle clickable links within the paragraph
                self.handle_link_interactions(ui, spans);
            }

            DocNode::CodeBlock { lang, code } => {
                self.render_code_block(ui, lang, code, highlighter);
            }

            DocNode::BlockQuote(children) => {
                let left_bar_color = self.palette.quote_border;
                let bg_color = self.palette.quote_bg;

                Frame::none()
                    .fill(bg_color)
                    .inner_margin(Margin {
                        left: 14.0,
                        right: 12.0,
                        top: 8.0,
                        bottom: 8.0,
                    })
                    .show(ui, |ui| {
                        let min_pos = ui.min_rect().min;
                        let max_y = ui.min_rect().max.y;

                        for child in children {
                            self.render_node(ui, child, highlighter, images);
                            ui.add_space(4.0);
                        }

                        // Paint accent left line
                        let line_rect = Rect::from_min_max(
                            Pos2::new(min_pos.x + 2.0, min_pos.y),
                            Pos2::new(min_pos.x + 5.0, max_y.max(min_pos.y + 16.0)),
                        );
                        ui.painter().rect_filled(line_rect, Rounding::same(1.5), left_bar_color);
                    });
            }

            DocNode::Alert { kind, children } => {
                self.render_alert(ui, *kind, children, highlighter, images);
            }

            DocNode::List {
                ordered,
                start_num,
                items,
            } => {
                self.render_list(ui, *ordered, *start_num, items, highlighter, images, 0);
            }

            DocNode::Table {
                alignments,
                header,
                rows,
            } => {
                self.render_table(ui, alignments, header, rows);
            }

            DocNode::Rule => {
                ui.add_space(8.0);
                let (rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, Rounding::ZERO, self.palette.code_border);
                ui.add_space(8.0);
            }

            DocNode::Html(raw_html) => {
                ui.colored_label(self.palette.muted, raw_html);
            }
        }
    }

    fn render_image_block(
        &mut self,
        ui: &mut Ui,
        url: &str,
        title: &str,
        alt: &str,
        images: &mut ImageCache,
    ) {
        if let Some(texture) = images.get_or_load(ui.ctx(), self.base_dir, url) {
            let orig_size = texture.size_vec2();
            let avail_w = ui.available_width().max(50.0);
            let display_size = if orig_size.x > avail_w {
                let ratio = avail_w / orig_size.x;
                vec2(avail_w, orig_size.y * ratio)
            } else {
                orig_size
            };

            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(texture).fit_to_exact_size(display_size));
                if !title.is_empty() || !alt.is_empty() {
                    let caption = if !title.is_empty() { title } else { alt };
                    ui.colored_label(self.palette.muted, caption);
                }
            });
        } else {
            ui.colored_label(
                self.palette.muted,
                format!("[Image: {} ({})]", if alt.is_empty() { url } else { alt }, url),
            );
        }
    }

    fn render_alert(
        &mut self,
        ui: &mut Ui,
        kind: AlertKind,
        children: &[DocNode],
        highlighter: &SyntaxHighlighter,
        images: &mut ImageCache,
    ) {
        let border_color = self.palette.alert_border(kind);
        let bg_color = self.palette.alert_bg(kind);
        let title_color = self.palette.alert_title_color(kind);

        Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, self.palette.code_border))
            .inner_margin(Margin {
                left: 16.0,
                right: 14.0,
                top: 10.0,
                bottom: 10.0,
            })
            .show(ui, |ui| {
                let min_pos = ui.min_rect().min;
                let max_y = ui.min_rect().max.y;

                ui.horizontal(|ui| {
                    ui.colored_label(
                        title_color,
                        egui::RichText::new(format!("[{}]", kind.tag_label()))
                            .strong()
                            .size(self.font_size * 0.95),
                    );
                });
                ui.add_space(4.0);

                for child in children {
                    self.render_node(ui, child, highlighter, images);
                    ui.add_space(3.0);
                }

                let line_rect = Rect::from_min_max(
                    Pos2::new(min_pos.x, min_pos.y),
                    Pos2::new(min_pos.x + 4.0, max_y),
                );
                ui.painter().rect_filled(
                    line_rect,
                    Rounding {
                        nw: 6.0,
                        ne: 0.0,
                        sw: 6.0,
                        se: 0.0,
                    },
                    border_color,
                );
            });
    }

    fn render_code_block(
        &mut self,
        ui: &mut Ui,
        lang: &str,
        code: &str,
        highlighter: &SyntaxHighlighter,
    ) {
        let code_job = highlighter.highlight_to_job(
            code,
            lang,
            self.palette.syntect_theme,
            self.font_size * 0.92,
            self.palette.text,
        );

        let code_str = code.to_string();

        Frame::none()
            .fill(self.palette.code_bg)
            .stroke(Stroke::new(1.0, self.palette.code_border))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                // Header banner
                ui.horizontal(|ui| {
                    let display_lang = if lang.is_empty() { "text" } else { lang };
                    ui.colored_label(self.palette.muted, display_lang);

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Copy").on_hover_text("Copy code to clipboard").clicked() {
                            self.events.push(RenderEvent::CopyToClipboard(code_str.clone()));
                        }
                        if ui.button("Explain").on_hover_text("Ask AI Assistant to explain this code").clicked() {
                            self.events.push(RenderEvent::ExplainCode {
                                lang: lang.to_string(),
                                code: code_str,
                            });
                        }
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Code scroll area
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.label(code_job);
                    });
            });
    }

    fn render_list(
        &mut self,
        ui: &mut Ui,
        ordered: bool,
        start_num: u64,
        items: &[ListItem],
        highlighter: &SyntaxHighlighter,
        images: &mut ImageCache,
        indent_level: usize,
    ) {
        for (i, item) in items.iter().enumerate() {
            ui.horizontal(|ui| {
                let indent = (indent_level as f32) * 20.0 + 8.0;
                ui.add_space(indent);

                if let Some(checked) = item.checkbox {
                    let mark = if checked { "[x] " } else { "[ ] " };
                    ui.colored_label(self.palette.accent, mark);
                } else if ordered {
                    let num = start_num + (i as u64);
                    ui.colored_label(self.palette.muted, format!("{}.", num));
                } else {
                    ui.colored_label(self.palette.accent, "-");
                }

                ui.vertical(|ui| {
                    for child in &item.children {
                        self.render_node(ui, child, highlighter, images);
                    }
                });
            });
            ui.add_space(3.0);
        }
    }

    fn render_table(
        &mut self,
        ui: &mut Ui,
        alignments: &[Alignment],
        header: &TableRow,
        rows: &[TableRow],
    ) {
        let num_cols = header
            .cells
            .len()
            .max(rows.iter().map(|r| r.cells.len()).max().unwrap_or(0));
        if num_cols == 0 {
            return;
        }

        Frame::none()
            .stroke(Stroke::new(1.0, self.palette.table_border))
            .rounding(Rounding::same(4.0))
            .show(ui, |ui| {
                egui::Grid::new(ui.next_auto_id())
                    .striped(true)
                    .spacing(vec2(16.0, 8.0))
                    .show(ui, |ui| {
                        // Header
                        for cell in &header.cells {
                            let mut job = LayoutJob::default();
                            self.append_spans_to_job(
                                &mut job,
                                &cell.spans,
                                self.font_size * 0.95,
                                self.palette.heading_colors[1],
                                true,
                                false,
                                false,
                            );
                            ui.label(job);
                        }
                        ui.end_row();

                        // Rows
                        for row in rows {
                            for (c_idx, cell) in row.cells.iter().enumerate() {
                                let _align = alignments.get(c_idx).copied().unwrap_or(Alignment::Left);
                                let mut job = LayoutJob::default();
                                self.append_spans_to_job(
                                    &mut job,
                                    &cell.spans,
                                    self.font_size * 0.95,
                                    self.palette.text,
                                    false,
                                    false,
                                    false,
                                );
                                ui.label(job);
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    fn append_spans_to_job(
        &self,
        job: &mut LayoutJob,
        spans: &[InlineSpan],
        font_size: f32,
        base_color: Color32,
        is_bold: bool,
        is_italic: bool,
        is_strikethrough: bool,
    ) {
        for span in spans {
            match span {
                InlineSpan::Text(t) => {
                    self.append_text_with_search(
                        job,
                        t,
                        font_size,
                        base_color,
                        false,
                        is_bold,
                        is_italic,
                        is_strikethrough,
                    );
                }
                InlineSpan::Bold(children) => {
                    self.append_spans_to_job(
                        job,
                        children,
                        font_size,
                        base_color,
                        true,
                        is_italic,
                        is_strikethrough,
                    );
                }
                InlineSpan::Italic(children) => {
                    self.append_spans_to_job(
                        job,
                        children,
                        font_size,
                        base_color,
                        is_bold,
                        true,
                        is_strikethrough,
                    );
                }
                InlineSpan::Strikethrough(children) => {
                    self.append_spans_to_job(
                        job,
                        children,
                        font_size,
                        base_color,
                        is_bold,
                        is_italic,
                        true,
                    );
                }
                InlineSpan::Code(code) => {
                    job.append(
                        code,
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(font_size * 0.9),
                            color: self.palette.code_inline_text,
                            background: self.palette.code_inline_bg,
                            ..Default::default()
                        },
                    );
                }
                InlineSpan::Link { text, .. } => {
                    self.append_spans_to_job(
                        job,
                        text,
                        font_size,
                        self.palette.link,
                        is_bold,
                        is_italic,
                        is_strikethrough,
                    );
                }
                InlineSpan::Image { alt, .. } => {
                    job.append(
                        &format!("[Image: {}]", alt),
                        0.0,
                        TextFormat {
                            font_id: FontId::proportional(font_size * 0.9),
                            color: self.palette.muted,
                            ..Default::default()
                        },
                    );
                }
                InlineSpan::SoftBreak => {
                    job.append(
                        " ",
                        0.0,
                        TextFormat {
                            font_id: FontId::proportional(font_size),
                            color: base_color,
                            ..Default::default()
                        },
                    );
                }
                InlineSpan::HardBreak => {
                    job.append(
                        "\n",
                        0.0,
                        TextFormat {
                            font_id: FontId::proportional(font_size),
                            color: base_color,
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    fn append_text_with_search(
        &self,
        job: &mut LayoutJob,
        text: &str,
        font_size: f32,
        color: Color32,
        is_monospace: bool,
        _is_bold: bool,
        is_italic: bool,
        is_strikethrough: bool,
    ) {
        let font_id = if is_monospace {
            FontId::monospace(font_size)
        } else {
            FontId::proportional(font_size)
        };

        if self.search_query.trim().is_empty() {
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id,
                    color,
                    italics: is_italic,
                    strikethrough: if is_strikethrough {
                        Stroke::new(1.0, color)
                    } else {
                        Stroke::NONE
                    },
                    ..Default::default()
                },
            );
            return;
        }

        let query = self.search_query.to_lowercase();
        let text_lower = text.to_lowercase();
        let mut last_idx = 0;

        for (match_start, _) in text_lower.match_indices(&query) {
            if match_start > last_idx {
                job.append(
                    &text[last_idx..match_start],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        italics: is_italic,
                        strikethrough: if is_strikethrough {
                            Stroke::new(1.0, color)
                        } else {
                            Stroke::NONE
                        },
                        ..Default::default()
                    },
                );
            }

            let match_end = match_start + query.len();
            job.append(
                &text[match_start..match_end],
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Color32::BLACK,
                    background: self.palette.search_match_bg,
                    italics: is_italic,
                    strikethrough: if is_strikethrough {
                        Stroke::new(1.0, Color32::BLACK)
                    } else {
                        Stroke::NONE
                    },
                    ..Default::default()
                },
            );

            last_idx = match_end;
        }

        if last_idx < text.len() {
            job.append(
                &text[last_idx..],
                0.0,
                TextFormat {
                    font_id,
                    color,
                    italics: is_italic,
                    strikethrough: if is_strikethrough {
                        Stroke::new(1.0, color)
                    } else {
                        Stroke::NONE
                    },
                    ..Default::default()
                },
            );
        }
    }

    fn handle_link_interactions(&mut self, ui: &mut Ui, spans: &[InlineSpan]) {
        for span in spans {
            if let InlineSpan::Link { url, text, .. } = span {
                let link_text = text.iter().map(|s| s.plain_text()).collect::<String>();
                let btn_text = format!("[link] {}", link_text);
                if ui.small_button(btn_text).clicked() {
                    if url.starts_with("http://") || url.starts_with("https://") {
                        self.events.push(RenderEvent::OpenLink(url.clone()));
                    } else if url.ends_with(".md") {
                        let path = ImageCache::resolve_path(self.base_dir, url);
                        self.events.push(RenderEvent::OpenFilePath(path));
                    } else {
                        self.events.push(RenderEvent::OpenLink(url.clone()));
                    }
                }
            }
        }
    }
}
