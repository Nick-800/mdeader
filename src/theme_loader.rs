use crate::theme::ThemePalette;
use egui::Color32;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct CustomThemeFile {
    pub name: String,
    pub id: String,
    #[serde(default = "default_is_dark")]
    pub is_dark: bool,
    #[serde(default)]
    pub syntect_theme: Option<String>,
    pub colors: ThemeColorsToml,
}

fn default_is_dark() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeColorsToml {
    pub bg: Vec<u8>,
    pub panel_bg: Vec<u8>,
    pub subtle_bg: Vec<u8>,
    pub text: Vec<u8>,
    pub muted: Vec<u8>,
    pub accent: Vec<u8>,
    pub link: Vec<u8>,
    pub code_bg: Vec<u8>,
    pub code_border: Vec<u8>,
    pub code_inline_bg: Vec<u8>,
    pub code_inline_text: Vec<u8>,
    pub quote_border: Vec<u8>,
    pub quote_bg: Vec<u8>,
    pub table_header_bg: Vec<u8>,
    pub table_alt_bg: Vec<u8>,
    pub table_border: Vec<u8>,
    pub search_match_bg: Vec<u8>,
    pub active_search_match_bg: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CustomTheme {
    pub name: String,
    pub id: String,
    pub is_dark: bool,
    pub palette: ThemePalette,
}

pub struct ThemeLoader;

impl ThemeLoader {
    pub fn themes_dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "mdeader", "mdeader").map(|dirs| {
            dirs.config_dir().join("themes")
        })
    }

    /// Loads all valid custom .toml theme files from the themes directory.
    pub fn load_all() -> Vec<CustomTheme> {
        let mut custom_themes = Vec::new();
        let Some(dir) = Self::themes_dir() else {
            return custom_themes;
        };

        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
            Self::write_sample_theme(&dir);
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            return custom_themes;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(theme) = Self::load_file(&path) {
                    custom_themes.push(theme);
                }
            }
        }

        custom_themes
    }

    pub fn load_file(path: &Path) -> Result<CustomTheme, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read theme file: {}", e))?;
        let parsed: CustomThemeFile = toml::from_str(&content)
            .map_err(|e| format!("Invalid theme TOML: {}", e))?;

        let parse_color = |vec: &[u8]| -> Color32 {
            match vec.len() {
                3 => Color32::from_rgb(vec[0], vec[1], vec[2]),
                4 => Color32::from_rgba_premultiplied(vec[0], vec[1], vec[2], vec[3]),
                _ => Color32::from_rgb(128, 128, 128),
            }
        };

        let c = &parsed.colors;
        let accent = parse_color(&c.accent);
        let syntect_theme: &'static str = match parsed.syntect_theme {
            Some(t) => Box::leak(t.into_boxed_str()),
            None => {
                if parsed.is_dark {
                    "base16-ocean.dark"
                } else {
                    "base16-ocean.light"
                }
            }
        };

        let palette = ThemePalette {
            bg: parse_color(&c.bg),
            panel_bg: parse_color(&c.panel_bg),
            subtle_bg: parse_color(&c.subtle_bg),
            text: parse_color(&c.text),
            muted: parse_color(&c.muted),
            heading_colors: [accent, accent, accent, accent, accent, accent],
            accent,
            link: parse_color(&c.link),
            code_bg: parse_color(&c.code_bg),
            code_border: parse_color(&c.code_border),
            code_inline_bg: parse_color(&c.code_inline_bg),
            code_inline_text: parse_color(&c.code_inline_text),
            quote_border: parse_color(&c.quote_border),
            quote_bg: parse_color(&c.quote_bg),
            table_header_bg: parse_color(&c.table_header_bg),
            table_alt_bg: parse_color(&c.table_alt_bg),
            table_border: parse_color(&c.table_border),
            search_match_bg: parse_color(&c.search_match_bg),
            active_search_match_bg: parse_color(&c.active_search_match_bg),
            syntect_theme,
        };

        Ok(CustomTheme {
            name: parsed.name,
            id: parsed.id,
            is_dark: parsed.is_dark,
            palette,
        })
    }

    fn write_sample_theme(dir: &Path) {
        let sample_path = dir.join("tokyo_night.toml");
        if !sample_path.exists() {
            let sample_toml = r#"name = "Tokyo Night"
id = "tokyonight"
is_dark = true

[colors]
bg = [26, 27, 38]
panel_bg = [36, 40, 59]
subtle_bg = [41, 46, 66]
text = [192, 202, 245]
muted = [86, 95, 137]
accent = [122, 162, 247]
link = [122, 162, 247]
code_bg = [31, 35, 53]
code_border = [65, 72, 104]
code_inline_bg = [41, 46, 66]
code_inline_text = [255, 158, 100]
quote_border = [122, 162, 247]
quote_bg = [36, 40, 59, 80]
table_header_bg = [36, 40, 59]
table_alt_bg = [26, 27, 38]
table_border = [65, 72, 104]
search_match_bg = [224, 175, 104, 120]
active_search_match_bg = [255, 158, 100, 180]
"#;
            let _ = std::fs::write(sample_path, sample_toml);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_custom_theme() {
        let toml_str = r#"name = "Sample"
id = "sample"
is_dark = true

[colors]
bg = [10, 10, 10]
panel_bg = [20, 20, 20]
subtle_bg = [30, 30, 30]
text = [200, 200, 200]
muted = [100, 100, 100]
accent = [50, 150, 250]
link = [50, 150, 250]
code_bg = [15, 15, 15]
code_border = [40, 40, 40]
code_inline_bg = [25, 25, 25]
code_inline_text = [220, 120, 40]
quote_border = [50, 150, 250]
quote_bg = [50, 150, 250, 30]
table_header_bg = [20, 20, 20]
table_alt_bg = [15, 15, 15]
table_border = [40, 40, 40]
search_match_bg = [200, 150, 50, 100]
active_search_match_bg = [240, 120, 40, 180]
"#;
        let parsed: CustomThemeFile = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.name, "Sample");
        assert_eq!(parsed.id, "sample");
        assert!(parsed.is_dark);
    }
}
