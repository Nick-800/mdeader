use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct SyntaxHighlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        Self {
            syntax_set,
            theme_set,
        }
    }

    pub fn highlight_to_job(
        &self,
        code: &str,
        lang: &str,
        theme_name: &str,
        font_size: f32,
        default_color: Color32,
    ) -> LayoutJob {
        let syntax = if lang.is_empty() {
            self.syntax_set.find_syntax_plain_text()
        } else {
            self.syntax_set
                .find_syntax_by_token(lang)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
        };

        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .or_else(|| self.theme_set.themes.get("base16-ocean.dark"))
            .or_else(|| self.theme_set.themes.values().next());

        let font_id = FontId::monospace(font_size);
        let mut job = LayoutJob::default();

        let Some(theme) = theme else {
            job.append(
                code,
                0.0,
                TextFormat {
                    font_id,
                    color: default_color,
                    ..Default::default()
                },
            );
            return job;
        };

        let mut highlighter = HighlightLines::new(syntax, theme);

        for line in code.split_inclusive('\n') {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        let fg = Color32::from_rgba_premultiplied(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                            style.foreground.a,
                        );
                        job.append(
                            text,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color: fg,
                                ..Default::default()
                            },
                        );
                    }
                }
                Err(_) => {
                    job.append(
                        line,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: default_color,
                            ..Default::default()
                        },
                    );
                }
            }
        }

        job
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let job = highlighter.highlight_to_job(
            "fn main() {\n    println!(\"Hello\");\n}",
            "rust",
            "base16-ocean.dark",
            14.0,
            Color32::WHITE,
        );
        assert!(!job.text.is_empty());
    }
}
