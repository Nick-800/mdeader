use crate::document::HeadingItem;
use crate::theme::ThemePalette;
use egui::{Align, Layout, Response, ScrollArea, TextEdit, Ui};

pub struct TocView {
    pub filter: String,
}

impl Default for TocView {
    fn default() -> Self {
        Self::new()
    }
}

impl TocView {
    pub fn new() -> Self {
        Self {
            filter: String::new(),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        headings: &[HeadingItem],
        palette: &ThemePalette,
        active_heading_index: Option<usize>,
    ) -> Option<usize> {
        let mut clicked_heading = None;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Outline");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !self.filter.is_empty() && ui.small_button("x").on_hover_text("Clear filter").clicked() {
                        self.filter.clear();
                    }
                });
            });

            ui.add_space(4.0);

            ui.add(
                TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter headings...")
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            if headings.is_empty() {
                ui.colored_label(palette.muted, "No headings found");
                return;
            }

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for h in headings {
                        if !self.filter.is_empty()
                            && !h.title.to_lowercase().contains(&self.filter.to_lowercase())
                        {
                            continue;
                        }

                        let indent = ((h.level.saturating_sub(1)) as f32) * 14.0;
                        let is_active = active_heading_index == Some(h.index);

                        ui.horizontal(|ui| {
                            if indent > 0.0 {
                                ui.add_space(indent);
                            }

                            let heading_color = if is_active {
                                palette.accent
                            } else {
                                let lvl_idx = (h.level.saturating_sub(1) as usize).min(5);
                                palette.heading_colors[lvl_idx]
                            };

                            let prefix = match h.level {
                                1 => "H1",
                                2 => "H2",
                                3 => "H3",
                                4 => "H4",
                                5 => "H5",
                                _ => "H6",
                            };

                            ui.colored_label(palette.muted, prefix);

                            let text = egui::RichText::new(&h.title)
                                .color(heading_color)
                                .size(13.0);

                            let response: Response = ui.selectable_label(is_active, text);
                            if response.clicked() {
                                clicked_heading = Some(h.index);
                            }
                        });
                    }
                });
        });

        clicked_heading
    }
}
