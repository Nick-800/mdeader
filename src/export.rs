use crate::theme::ThemePalette;
use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
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
        let events: Vec<Event> = parser.collect();
        let mut transformed_events = Vec::new();
        let mut idx = 0;
        let mut alert_depth: Vec<bool> = Vec::new();

        while idx < events.len() {
            match &events[idx] {
                Event::Start(Tag::BlockQuote(_)) => {
                    let mut is_alert = None;

                    if idx + 1 < events.len() && matches!(events[idx + 1], Event::Start(Tag::Paragraph)) {
                        if let Some(Event::Text(t)) = events.get(idx + 2) {
                            let upper = t.trim_start().to_ascii_uppercase();
                            for (kind_class, title, tag) in [
                                ("note", "Note", "[!NOTE]"),
                                ("tip", "Tip", "[!TIP]"),
                                ("important", "Important", "[!IMPORTANT]"),
                                ("warning", "Warning", "[!WARNING]"),
                                ("caution", "Caution", "[!CAUTION]"),
                            ] {
                                if upper.starts_with(tag) {
                                    let rem = t.trim_start()[tag.len()..].trim_start();
                                    is_alert = Some((kind_class, title, if rem.is_empty() { None } else { Some(rem.to_string()) }, idx + 3));
                                    break;
                                }
                            }
                        }

                        if is_alert.is_none() && idx + 4 < events.len() {
                            if let (Some(Event::Text(t0)), Some(Event::Text(t1)), Some(Event::Text(t2))) =
                                (events.get(idx + 2), events.get(idx + 3), events.get(idx + 4))
                            {
                                if t0.trim_start() == "[" {
                                    let tag_upper = t1.trim().to_ascii_uppercase();
                                    let detected = match tag_upper.as_str() {
                                        "!NOTE" => Some(("note", "Note")),
                                        "!TIP" => Some(("tip", "Tip")),
                                        "!IMPORTANT" => Some(("important", "Important")),
                                        "!WARNING" => Some(("warning", "Warning")),
                                        "!CAUTION" => Some(("caution", "Caution")),
                                        _ => None,
                                    };
                                    if let Some((kind_class, title)) = detected {
                                        let t2_trim = t2.trim_start();
                                        if t2_trim.starts_with(']') {
                                            let rem = t2_trim[1..].trim_start();
                                            is_alert = Some((kind_class, title, if rem.is_empty() { None } else { Some(rem.to_string()) }, idx + 5));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((kind_class, title, remainder, new_idx)) = is_alert {
                        alert_depth.push(true);
                        transformed_events.push(Event::Html(format!("<div class=\"markdown-alert markdown-alert-{}\"><p class=\"markdown-alert-title\">{}</p>", kind_class, title).into()));
                        transformed_events.push(Event::Start(Tag::Paragraph));
                        let skip_to = if let Some(rem) = remainder {
                            transformed_events.push(Event::Text(rem.into()));
                            new_idx
                        } else if let Some(Event::SoftBreak) | Some(Event::HardBreak) = events.get(new_idx) {
                            new_idx + 1
                        } else {
                            new_idx
                        };
                        idx = skip_to;
                        continue;
                    } else {
                        alert_depth.push(false);
                        transformed_events.push(events[idx].clone());
                    }
                }
                Event::End(TagEnd::BlockQuote) => {
                    if let Some(true) = alert_depth.pop() {
                        transformed_events.push(Event::Html("</div>".into()));
                    } else {
                        transformed_events.push(events[idx].clone());
                    }
                }
                _ => {
                    transformed_events.push(events[idx].clone());
                }
            }
            idx += 1;
        }

        let mut html_output = String::new();
        html::push_html(&mut html_output, transformed_events.into_iter());

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

    #[test]
    fn test_export_alert_to_html() {
        let md = "> [!NOTE]\n> Note content here.";
        let palette = ThemeKind::GitHubDark.palette();
        let html = HtmlExporter::export_to_html(md, "Alert Doc", &palette);

        assert!(html.contains("markdown-alert-note"));
        assert!(html.contains("markdown-alert-title"));
        assert!(html.contains("Note content here."));
    }
}
