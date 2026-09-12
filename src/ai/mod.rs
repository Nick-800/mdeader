pub mod client;
pub mod context;
pub mod view;

use crate::config::AiConfig;
use client::AiClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    #[allow(dead_code)]
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

pub enum AiEvent {
    Token(String),
    Done,
    Error(String),
}

pub struct AiManager {
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    pub is_generating: bool,
    pub error_message: Option<String>,
    pub cancel_token: Arc<AtomicBool>,
    rx: Option<Receiver<AiEvent>>,
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AiManager {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            is_generating: false,
            error_message: None,
            cancel_token: Arc::new(AtomicBool::new(false)),
            rx: None,
        }
    }

    /// Polls incoming events from background AI thread and requests UI repaint if tokens arrive.
    pub fn update(&mut self, ctx: &egui::Context) {
        if let Some(ref rx) = self.rx {
            let mut got_tokens = false;
            while let Ok(event) = rx.try_recv() {
                match event {
                    AiEvent::Token(token) => {
                        got_tokens = true;
                        if let Some(last) = self.messages.last_mut() {
                            if last.role == MessageRole::Assistant {
                                last.content.push_str(&token);
                            } else {
                                self.messages.push(ChatMessage {
                                    role: MessageRole::Assistant,
                                    content: token,
                                });
                            }
                        } else {
                            self.messages.push(ChatMessage {
                                role: MessageRole::Assistant,
                                content: token,
                            });
                        }
                    }
                    AiEvent::Done => {
                        self.is_generating = false;
                        self.rx = None;
                        ctx.request_repaint();
                        break;
                    }
                    AiEvent::Error(err) => {
                        self.is_generating = false;
                        self.error_message = Some(err);
                        self.rx = None;
                        ctx.request_repaint();
                        break;
                    }
                }
            }

            if got_tokens {
                ctx.request_repaint();
            }
        }
    }

    /// Stops any in-flight generation.
    pub fn stop_generation(&mut self) {
        self.cancel_token.store(true, Ordering::Relaxed);
        self.is_generating = false;
        self.rx = None;
    }

    /// Clears the current chat history and error state.
    pub fn clear(&mut self) {
        self.stop_generation();
        self.messages.clear();
        self.error_message = None;
    }

    /// Dispatches an AI prompt to background worker thread.
    pub fn send_prompt(
        &mut self,
        config: &AiConfig,
        system_prompt: String,
        user_prompt: String,
        ctx: egui::Context,
    ) {
        if user_prompt.trim().is_empty() {
            return;
        }

        self.stop_generation();
        self.error_message = None;

        // Record user message
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: user_prompt.clone(),
        });

        // Reserve assistant slot
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: String::new(),
        });

        let (tx, rx): (Sender<AiEvent>, Receiver<AiEvent>) = mpsc::channel();
        self.rx = Some(rx);
        self.is_generating = true;

        let cancel_token = Arc::new(AtomicBool::new(false));
        self.cancel_token = Arc::clone(&cancel_token);

        let config_clone = config.clone();

        thread::spawn(move || {
            let res = AiClient::stream_query(
                &config_clone,
                &system_prompt,
                &user_prompt,
                Arc::clone(&cancel_token),
                |token| {
                    if cancel_token.load(Ordering::Relaxed) {
                        return false;
                    }
                    let _ = tx.send(AiEvent::Token(token.to_string()));
                    ctx.request_repaint();
                    true
                },
            );

            match res {
                Ok(_) => {
                    let _ = tx.send(AiEvent::Done);
                }
                Err(err) => {
                    let _ = tx.send(AiEvent::Error(err));
                }
            }
            ctx.request_repaint();
        });
    }

    /// Quick action: Explain code block.
    pub fn explain_code(
        &mut self,
        config: &AiConfig,
        system_prompt: String,
        lang: &str,
        code: &str,
        ctx: egui::Context,
    ) {
        let prompt = format!(
            "Please explain this {} code block. Describe its purpose, how it works, and key components:\n\n```{}\n{}\n```",
            if lang.is_empty() { "code" } else { lang },
            lang,
            code
        );
        self.send_prompt(config, system_prompt, prompt, ctx);
    }

    /// Quick action: Summarize document.
    pub fn summarize_document(
        &mut self,
        config: &AiConfig,
        system_prompt: String,
        doc_context: String,
        ctx: egui::Context,
    ) {
        let prompt = format!(
            "Please provide a concise executive summary and key takeaways of this document:\n\n{}",
            doc_context
        );
        self.send_prompt(config, system_prompt, prompt, ctx);
    }

    /// Quick action: Extract action items.
    pub fn extract_action_items(
        &mut self,
        config: &AiConfig,
        system_prompt: String,
        doc_context: String,
        ctx: egui::Context,
    ) {
        let prompt = format!(
            "Extract all actionable tasks, to-dos, checklists, and next steps mentioned in this document into a structured task list:\n\n{}",
            doc_context
        );
        self.send_prompt(config, system_prompt, prompt, ctx);
    }
}
