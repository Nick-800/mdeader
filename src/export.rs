use crate::theme::ThemePalette;
use pulldown_cmark::{html, Options, Parser};
use std::path::Path;

pub struct HtmlExporter;

impl HtmlExporter {
    pub fn export_to_html(
        raw_markdown: &str,
        title: &str,
        palette: &ThemePalette,
    ) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(raw_markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        let css = palette.to_css();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{css}
    </style>
</head>
<body>
{html_output}
</body>
</html>"#,
            title = html_escape(title),
            css = css,
            html_output = html_output
        )
    }

    pub fn save_to_file(
        raw_markdown: &str,
        title: &str,
        palette: &ThemePalette,
        dest_path: &Path,
    ) -> Result<(), std::io::Error> {
        let html = Self::export_to_html(raw_markdown, title, palette);
        std::fs::write(dest_path, html)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeKind;

    #[test]
    fn test_export_to_html() {
        let md = "# Welcome to mdeader\n\nThis is a **test** document.";
        let palette = ThemeKind::GitHubDark.palette();
        let html = HtmlExporter::export_to_html(md, "Test Doc", &palette);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Doc</title>"));
        assert!(html.contains("<h1>Welcome to mdeader</h1>"));
        assert!(html.contains("<strong>test</strong>"));
        assert!(html.contains("background-color:"));
    }
}
