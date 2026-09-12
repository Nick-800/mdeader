use crate::document::Document;
use std::path::Path;

pub struct DocumentContext;

impl DocumentContext {
    pub const MAX_CONTEXT_CHARS: usize = 28_000;

    /// Builds a system instruction informing the AI assistant about the document and environment rules.
    pub fn build_system_prompt(file_path: Option<&Path>, doc: Option<&Document>) -> String {
        let file_name = file_path
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("active document");

        let mut prompt = String::new();
        prompt.push_str("You are mdeader's native Markdown reading assistant.\n");
        prompt.push_str("Your role is to assist the user by explaining, summarizing, and answering questions about the open Markdown document.\n");
        prompt.push_str("Formatting rules:\n");
        prompt.push_str("- Format your responses in clean GitHub-flavored Markdown.\n");
        prompt.push_str("- Do not use emojis in any output or responses.\n");
        prompt.push_str("- Be concise, clear, and direct.\n\n");

        prompt.push_str(&format!("Active document: {}\n", file_name));

        if let Some(d) = doc {
            if !d.headings.is_empty() {
                prompt.push_str("\nDocument Outline / Table of Contents:\n");
                for h in &d.headings {
                    let indent = "  ".repeat(h.level.saturating_sub(1) as usize);
                    prompt.push_str(&format!("{}- {}\n", indent, h.title));
                }
            }
        }

        prompt
    }

    /// Extracts context from the document, optionally focused on a specific heading index.
    pub fn extract_document_context(
        doc: Option<&Document>,
        target_heading_idx: Option<usize>,
    ) -> String {
        let Some(d) = doc else {
            return "No document is currently open.".to_string();
        };

        if let Some(idx) = target_heading_idx {
            if let Some(h) = d.headings.get(idx) {
                // Focus context on heading
                let heading_title = &h.title;
                let raw_content = Self::truncate_safe(&d.raw, Self::MAX_CONTEXT_CHARS);
                return format!(
                    "Focus Section: {}\n\nFull Document Content:\n```markdown\n{}\n```",
                    heading_title, raw_content
                );
            }
        }

        let raw_content = Self::truncate_safe(&d.raw, Self::MAX_CONTEXT_CHARS);
        format!("Document Content:\n```markdown\n{}\n```", raw_content)
    }

    /// Safely truncates context text to max_chars without splitting UTF-8 characters.
    pub fn truncate_safe(text: &str, max_chars: usize) -> String {
        if text.chars().count() <= max_chars {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(max_chars).collect();
            format!("{}\n\n[... document truncated for context length ...]", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_safe() {
        let short = "Hello world";
        assert_eq!(DocumentContext::truncate_safe(short, 20), "Hello world");

        let long = "a".repeat(100);
        let truncated = DocumentContext::truncate_safe(&long, 50);
        assert!(truncated.starts_with(&"a".repeat(50)));
        assert!(truncated.contains("truncated for context length"));
    }

    #[test]
    fn test_build_system_prompt() {
        let content = "# Title\n\n## Section 1\nContent 1\n\n## Section 2\nContent 2";
        let doc = Document::parse(content);
        let prompt = DocumentContext::build_system_prompt(Some(Path::new("test.md")), Some(&doc));
        assert!(prompt.contains("mdeader's native Markdown reading assistant"));
        assert!(prompt.contains("Do not use emojis"));
        assert!(prompt.contains("Section 1"));
        assert!(prompt.contains("Section 2"));
    }
}
