use super::context::DocumentContext;
use super::{AiManager, MessageRole};
use crate::config::Config;
use crate::document::Document;
use crate::theme::ThemePalette;
use eframe::egui::{self, Align, Frame, Key, Layout, Margin, Rounding, ScrollArea, Stroke, TextEdit, Ui};
use std::path::Path;

pub struct AiView;

impl AiView {
    pub fn render(
        ui: &mut Ui,
        ai: &mut AiManager,
        config: &mut Config,
        current_path: Option<&Path>,
        document: Option<&Document>,
        target_heading_idx: Option<usize>,
        palette: &ThemePalette,
    ) {
        let ctx = ui.ctx().clone();

        // 1. Header
        ui.horizontal(|ui| {
            ui.strong("AI Assistant");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("x").on_hover_text("Close AI Drawer (Ctrl+Shift+A)").clicked() {
                    config.show_ai = false;
                    let _ = config.save();
                }

                if ui.small_button("Clear").on_hover_text("Clear chat history").clicked() {
                    ai.clear();
                }

                if ai.is_generating {
                    if ui.button("Stop").on_hover_text("Stop generation").clicked() {
                        ai.stop_generation();
                    }
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Provider and Model Settings (collapsible header)
        ui.collapsing("Configuration", |ui| {
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Provider:");
                let current_prov = config.ai.provider.clone();
                egui::ComboBox::from_id_source("ai_provider_selector")
                    .selected_text(&config.ai.provider)
                    .show_ui(ui, |ui| {
                        for p in &["ollama", "gemini", "custom"] {
                            if ui.selectable_value(&mut config.ai.provider, p.to_string(), *p).clicked() {
                                changed = true;
                            }
                        }
                    });
                if config.ai.provider != current_prov {
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Model:");
                let resp = ui.add(
                    TextEdit::singleline(&mut config.ai.model)
                        .hint_text("e.g. llama3.2, gemini-1.5-flash")
                        .desired_width(180.0),
                );
                if resp.changed() {
                    changed = true;
                }
            });

            if config.ai.provider != "gemini" {
                ui.horizontal(|ui| {
                    ui.label("Endpoint:");
                    let resp = ui.add(
                        TextEdit::singleline(&mut config.ai.endpoint)
                            .hint_text("http://localhost:11434")
                            .desired_width(180.0),
                    );
                    if resp.changed() {
                        changed = true;
                    }
                });
            }

            if config.ai.provider != "ollama" {
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    let mut key = config.ai.api_key.clone().unwrap_or_default();
                    let resp = ui.add(
                        TextEdit::singleline(&mut key)
                            .password(true)
                            .hint_text("Enter API key")
                            .desired_width(180.0),
                    );
                    if resp.changed() {
                        config.ai.api_key = if key.trim().is_empty() {
                            None
                        } else {
                            Some(key.trim().to_string())
                        };
                        changed = true;
                    }
                });
            }

            if changed {
                let _ = config.save();
            }
        });

        ui.add_space(4.0);

        // 3. Quick Action Buttons
        ui.horizontal_wrapped(|ui| {
            let doc_context = DocumentContext::extract_document_context(document, target_heading_idx);
            let system_prompt = DocumentContext::build_system_prompt(current_path, document);

            if ui.small_button("TL;DR Summary").on_hover_text("Summarize active document").clicked() {
                ai.summarize_document(&config.ai, system_prompt.clone(), doc_context.clone(), ctx.clone());
            }

            if ui.small_button("Action Items").on_hover_text("Extract tasks and checklists").clicked() {
                ai.extract_action_items(&config.ai, system_prompt.clone(), doc_context.clone(), ctx.clone());
            }

            if target_heading_idx.is_some() {
                if ui.small_button("Explain Section").on_hover_text("Explain current heading section").clicked() {
                    let prompt = format!(
                        "Explain this section of the document in depth:\n\n{}",
                        doc_context
                    );
                    ai.send_prompt(&config.ai, system_prompt, prompt, ctx.clone());
                }
            }
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // 4. Chat Messages Scroll Area
        let available_height = ui.available_height() - 75.0;
        ScrollArea::vertical()
            .max_height(available_height.max(120.0))
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if ai.messages.is_empty() {
                    ui.add_space(20.0);
                    ui.colored_label(palette.muted, "Ask questions about the open document, or click one of the quick actions above.");
                }

                for msg in &ai.messages {
                    ui.add_space(4.0);
                    match msg.role {
                        MessageRole::User => {
                            Frame::none()
                                .fill(palette.code_bg)
                                .stroke(Stroke::new(1.0, palette.code_border))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.strong("You");
                                    ui.add_space(2.0);
                                    ui.label(&msg.content);
                                });
                        }
                        MessageRole::Assistant => {
                            Frame::none()
                                .fill(palette.subtle_bg)
                                .stroke(Stroke::new(1.0, palette.code_border))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.colored_label(palette.accent, "mdeader assistant");
                                    ui.add_space(2.0);
                                    if msg.content.is_empty() && ai.is_generating {
                                        ui.colored_label(palette.muted, "Thinking...");
                                    } else {
                                        ui.label(&msg.content);
                                    }
                                });
                        }
                        MessageRole::System => {
                            ui.colored_label(palette.muted, &msg.content);
                        }
                    }
                }

                if let Some(ref err) = ai.error_message {
                    ui.add_space(6.0);
                    let err_border = palette.alert_border(crate::document::AlertKind::Caution);
                    let err_bg = palette.alert_bg(crate::document::AlertKind::Caution);
                    let err_title = palette.alert_title_color(crate::document::AlertKind::Caution);
                    Frame::none()
                        .fill(err_bg)
                        .stroke(Stroke::new(1.0, err_border))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.colored_label(err_title, format!("Error: {}", err));
                        });
                }
            });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // 5. Input TextEdit & Send Button
        ui.horizontal(|ui| {
            let text_edit = TextEdit::multiline(&mut ai.input_text)
                .hint_text("Ask about this document... (Enter to send)")
                .desired_rows(2)
                .desired_width(ui.available_width() - 55.0);

            let resp = ui.add(text_edit);

            let mut submit = false;
            if resp.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift) {
                submit = true;
            }

            if ui.button("Send").clicked() || submit {
                let prompt = ai.input_text.trim().to_string();
                if !prompt.is_empty() {
                    let doc_context = DocumentContext::extract_document_context(document, target_heading_idx);
                    let system_prompt = DocumentContext::build_system_prompt(current_path, document);
                    let full_prompt = format!("{}\n\nUser Question:\n{}", doc_context, prompt);

                    ai.input_text.clear();
                    ai.send_prompt(&config.ai, system_prompt, full_prompt, ctx);
                }
            }
        });
    }
}
