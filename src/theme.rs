use crate::document::AlertKind;
use egui::{Color32, Stroke, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeKind {
    GitHubDark,
    GitHubLight,
    CatppuccinMocha,
    CatppuccinLatte,
    Dracula,
    Nord,
    SolarizedDark,
    SolarizedLight,
}

impl ThemeKind {
    pub const ALL: &'static [ThemeKind] = &[
        ThemeKind::GitHubDark,
        ThemeKind::GitHubLight,
        ThemeKind::CatppuccinMocha,
        ThemeKind::CatppuccinLatte,
        ThemeKind::Dracula,
        ThemeKind::Nord,
        ThemeKind::SolarizedDark,
        ThemeKind::SolarizedLight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::GitHubDark => "GitHub Dark",
            ThemeKind::GitHubLight => "GitHub Light",
            ThemeKind::CatppuccinMocha => "Catppuccin Mocha",
            ThemeKind::CatppuccinLatte => "Catppuccin Latte",
            ThemeKind::Dracula => "Dracula",
            ThemeKind::Nord => "Nord",
            ThemeKind::SolarizedDark => "Solarized Dark",
            ThemeKind::SolarizedLight => "Solarized Light",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.to_lowercase().replace([' ', '-', '_'], "").as_str() {
            "githublight" | "ghlight" | "light" => ThemeKind::GitHubLight,
            "catppuccinmocha" | "mocha" => ThemeKind::CatppuccinMocha,
            "catppuccinlatte" | "latte" => ThemeKind::CatppuccinLatte,
            "dracula" => ThemeKind::Dracula,
            "nord" => ThemeKind::Nord,
            "solarizeddark" => ThemeKind::SolarizedDark,
            "solarizedlight" => ThemeKind::SolarizedLight,
            _ => ThemeKind::GitHubDark,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            ThemeKind::GitHubDark => "GitHubDark",
            ThemeKind::GitHubLight => "GitHubLight",
            ThemeKind::CatppuccinMocha => "CatppuccinMocha",
            ThemeKind::CatppuccinLatte => "CatppuccinLatte",
            ThemeKind::Dracula => "Dracula",
            ThemeKind::Nord => "Nord",
            ThemeKind::SolarizedDark => "SolarizedDark",
            ThemeKind::SolarizedLight => "SolarizedLight",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            ThemeKind::GitHubLight | ThemeKind::CatppuccinLatte | ThemeKind::SolarizedLight => false,
            _ => true,
        }
    }

    pub fn palette(&self) -> ThemePalette {
        match self {
            ThemeKind::GitHubDark => ThemePalette {
                bg: Color32::from_rgb(13, 17, 23),
                panel_bg: Color32::from_rgb(22, 27, 34),
                subtle_bg: Color32::from_rgb(33, 38, 45),
                text: Color32::from_rgb(230, 237, 243),
                muted: Color32::from_rgb(139, 148, 158),
                heading_colors: [
                    Color32::from_rgb(88, 166, 255),
                    Color32::from_rgb(121, 192, 255),
                    Color32::from_rgb(165, 214, 255),
                    Color32::from_rgb(201, 209, 217),
                    Color32::from_rgb(170, 179, 189),
                    Color32::from_rgb(139, 148, 158),
                ],
                accent: Color32::from_rgb(88, 166, 255),
                link: Color32::from_rgb(88, 166, 255),
                code_bg: Color32::from_rgb(22, 27, 34),
                code_border: Color32::from_rgb(48, 54, 61),
                code_inline_bg: Color32::from_rgb(33, 38, 45),
                code_inline_text: Color32::from_rgb(240, 136, 62),
                quote_border: Color32::from_rgb(56, 139, 253),
                quote_bg: Color32::from_rgba_premultiplied(56, 139, 253, 15),
                table_header_bg: Color32::from_rgb(22, 27, 34),
                table_alt_bg: Color32::from_rgb(18, 22, 28),
                table_border: Color32::from_rgb(48, 54, 61),
                search_match_bg: Color32::from_rgba_premultiplied(210, 153, 34, 100),
                active_search_match_bg: Color32::from_rgba_premultiplied(240, 136, 62, 180),
                syntect_theme: "base16-ocean.dark",
            },
            ThemeKind::GitHubLight => ThemePalette {
                bg: Color32::from_rgb(255, 255, 255),
                panel_bg: Color32::from_rgb(246, 248, 250),
                subtle_bg: Color32::from_rgb(234, 238, 242),
                text: Color32::from_rgb(31, 35, 40),
                muted: Color32::from_rgb(101, 109, 118),
                heading_colors: [
                    Color32::from_rgb(9, 105, 218),
                    Color32::from_rgb(15, 82, 186),
                    Color32::from_rgb(31, 35, 40),
                    Color32::from_rgb(50, 56, 62),
                    Color32::from_rgb(80, 87, 94),
                    Color32::from_rgb(101, 109, 118),
                ],
                accent: Color32::from_rgb(9, 105, 218),
                link: Color32::from_rgb(9, 105, 218),
                code_bg: Color32::from_rgb(246, 248, 250),
                code_border: Color32::from_rgb(208, 215, 222),
                code_inline_bg: Color32::from_rgb(234, 238, 242),
                code_inline_text: Color32::from_rgb(149, 56, 0),
                quote_border: Color32::from_rgb(9, 105, 218),
                quote_bg: Color32::from_rgba_premultiplied(9, 105, 218, 15),
                table_header_bg: Color32::from_rgb(246, 248, 250),
                table_alt_bg: Color32::from_rgb(250, 251, 252),
                table_border: Color32::from_rgb(208, 215, 222),
                search_match_bg: Color32::from_rgba_premultiplied(255, 223, 93, 160),
                active_search_match_bg: Color32::from_rgba_premultiplied(255, 140, 0, 180),
                syntect_theme: "InspiredGitHub",
            },
            ThemeKind::CatppuccinMocha => ThemePalette {
                bg: Color32::from_rgb(30, 30, 46),
                panel_bg: Color32::from_rgb(24, 24, 37),
                subtle_bg: Color32::from_rgb(49, 50, 68),
                text: Color32::from_rgb(205, 214, 244),
                muted: Color32::from_rgb(166, 173, 200),
                heading_colors: [
                    Color32::from_rgb(203, 166, 247), // Mauve
                    Color32::from_rgb(137, 180, 250), // Blue
                    Color32::from_rgb(148, 226, 213), // Teal
                    Color32::from_rgb(166, 227, 161), // Green
                    Color32::from_rgb(249, 226, 175), // Yellow
                    Color32::from_rgb(245, 194, 231), // Pink
                ],
                accent: Color32::from_rgb(203, 166, 247),
                link: Color32::from_rgb(137, 180, 250),
                code_bg: Color32::from_rgb(24, 24, 37),
                code_border: Color32::from_rgb(69, 71, 90),
                code_inline_bg: Color32::from_rgb(49, 50, 68),
                code_inline_text: Color32::from_rgb(250, 179, 135),
                quote_border: Color32::from_rgb(203, 166, 247),
                quote_bg: Color32::from_rgba_premultiplied(203, 166, 247, 18),
                table_header_bg: Color32::from_rgb(24, 24, 37),
                table_alt_bg: Color32::from_rgb(27, 27, 41),
                table_border: Color32::from_rgb(69, 71, 90),
                search_match_bg: Color32::from_rgba_premultiplied(249, 226, 175, 90),
                active_search_match_bg: Color32::from_rgba_premultiplied(250, 179, 135, 180),
                syntect_theme: "base16-mocha.dark",
            },
            ThemeKind::CatppuccinLatte => ThemePalette {
                bg: Color32::from_rgb(239, 241, 245),
                panel_bg: Color32::from_rgb(230, 233, 239),
                subtle_bg: Color32::from_rgb(204, 208, 218),
                text: Color32::from_rgb(76, 79, 105),
                muted: Color32::from_rgb(124, 127, 147),
                heading_colors: [
                    Color32::from_rgb(136, 57, 239), // Mauve
                    Color32::from_rgb(30, 102, 245),  // Blue
                    Color32::from_rgb(23, 146, 153),  // Teal
                    Color32::from_rgb(64, 160, 43),   // Green
                    Color32::from_rgb(223, 142, 29),  // Yellow
                    Color32::from_rgb(234, 118, 203), // Pink
                ],
                accent: Color32::from_rgb(136, 57, 239),
                link: Color32::from_rgb(30, 102, 245),
                code_bg: Color32::from_rgb(230, 233, 239),
                code_border: Color32::from_rgb(188, 192, 204),
                code_inline_bg: Color32::from_rgb(220, 224, 232),
                code_inline_text: Color32::from_rgb(254, 100, 11),
                quote_border: Color32::from_rgb(136, 57, 239),
                quote_bg: Color32::from_rgba_premultiplied(136, 57, 239, 18),
                table_header_bg: Color32::from_rgb(230, 233, 239),
                table_alt_bg: Color32::from_rgb(235, 237, 243),
                table_border: Color32::from_rgb(188, 192, 204),
                search_match_bg: Color32::from_rgba_premultiplied(223, 142, 29, 100),
                active_search_match_bg: Color32::from_rgba_premultiplied(254, 100, 11, 180),
                syntect_theme: "InspiredGitHub",
            },
            ThemeKind::Dracula => ThemePalette {
                bg: Color32::from_rgb(40, 42, 54),
                panel_bg: Color32::from_rgb(33, 34, 44),
                subtle_bg: Color32::from_rgb(68, 71, 90),
                text: Color32::from_rgb(248, 248, 242),
                muted: Color32::from_rgb(98, 114, 164),
                heading_colors: [
                    Color32::from_rgb(189, 147, 249), // Purple
                    Color32::from_rgb(139, 233, 253), // Cyan
                    Color32::from_rgb(80, 250, 123),  // Green
                    Color32::from_rgb(255, 184, 108), // Orange
                    Color32::from_rgb(255, 121, 198), // Pink
                    Color32::from_rgb(241, 250, 140), // Yellow
                ],
                accent: Color32::from_rgb(189, 147, 249),
                link: Color32::from_rgb(139, 233, 253),
                code_bg: Color32::from_rgb(33, 34, 44),
                code_border: Color32::from_rgb(68, 71, 90),
                code_inline_bg: Color32::from_rgb(68, 71, 90),
                code_inline_text: Color32::from_rgb(255, 121, 198),
                quote_border: Color32::from_rgb(189, 147, 249),
                quote_bg: Color32::from_rgba_premultiplied(189, 147, 249, 20),
                table_header_bg: Color32::from_rgb(33, 34, 44),
                table_alt_bg: Color32::from_rgb(36, 38, 48),
                table_border: Color32::from_rgb(68, 71, 90),
                search_match_bg: Color32::from_rgba_premultiplied(241, 250, 140, 100),
                active_search_match_bg: Color32::from_rgba_premultiplied(255, 184, 108, 180),
                syntect_theme: "base16-ocean.dark",
            },
            ThemeKind::Nord => ThemePalette {
                bg: Color32::from_rgb(46, 52, 64),
                panel_bg: Color32::from_rgb(36, 41, 51),
                subtle_bg: Color32::from_rgb(59, 66, 82),
                text: Color32::from_rgb(236, 239, 244),
                muted: Color32::from_rgb(147, 155, 172),
                heading_colors: [
                    Color32::from_rgb(136, 192, 208), // Frost 1
                    Color32::from_rgb(129, 161, 193), // Frost 2
                    Color32::from_rgb(94, 129, 172),  // Frost 3
                    Color32::from_rgb(163, 190, 140), // Green
                    Color32::from_rgb(235, 203, 139), // Yellow
                    Color32::from_rgb(180, 142, 173), // Purple
                ],
                accent: Color32::from_rgb(136, 192, 208),
                link: Color32::from_rgb(129, 161, 193),
                code_bg: Color32::from_rgb(36, 41, 51),
                code_border: Color32::from_rgb(59, 66, 82),
                code_inline_bg: Color32::from_rgb(59, 66, 82),
                code_inline_text: Color32::from_rgb(235, 203, 139),
                quote_border: Color32::from_rgb(136, 192, 208),
                quote_bg: Color32::from_rgba_premultiplied(136, 192, 208, 18),
                table_header_bg: Color32::from_rgb(36, 41, 51),
                table_alt_bg: Color32::from_rgb(41, 46, 57),
                table_border: Color32::from_rgb(59, 66, 82),
                search_match_bg: Color32::from_rgba_premultiplied(235, 203, 139, 100),
                active_search_match_bg: Color32::from_rgba_premultiplied(208, 135, 112, 180),
                syntect_theme: "base16-ocean.dark",
            },
            ThemeKind::SolarizedDark => ThemePalette {
                bg: Color32::from_rgb(0, 43, 54),
                panel_bg: Color32::from_rgb(7, 54, 66),
                subtle_bg: Color32::from_rgb(14, 63, 76),
                text: Color32::from_rgb(131, 148, 150),
                muted: Color32::from_rgb(88, 110, 117),
                heading_colors: [
                    Color32::from_rgb(38, 139, 210), // Blue
                    Color32::from_rgb(42, 161, 152), // Cyan
                    Color32::from_rgb(133, 153, 0),  // Green
                    Color32::from_rgb(181, 137, 0),  // Yellow
                    Color32::from_rgb(203, 75, 22),   // Orange
                    Color32::from_rgb(211, 54, 130), // Magenta
                ],
                accent: Color32::from_rgb(38, 139, 210),
                link: Color32::from_rgb(38, 139, 210),
                code_bg: Color32::from_rgb(7, 54, 66),
                code_border: Color32::from_rgb(14, 63, 76),
                code_inline_bg: Color32::from_rgb(14, 63, 76),
                code_inline_text: Color32::from_rgb(181, 137, 0),
                quote_border: Color32::from_rgb(38, 139, 210),
                quote_bg: Color32::from_rgba_premultiplied(38, 139, 210, 20),
                table_header_bg: Color32::from_rgb(7, 54, 66),
                table_alt_bg: Color32::from_rgb(3, 49, 61),
                table_border: Color32::from_rgb(14, 63, 76),
                search_match_bg: Color32::from_rgba_premultiplied(181, 137, 0, 100),
                active_search_match_bg: Color32::from_rgba_premultiplied(203, 75, 22, 180),
                syntect_theme: "Solarized (dark)",
            },
            ThemeKind::SolarizedLight => ThemePalette {
                bg: Color32::from_rgb(253, 246, 227),
                panel_bg: Color32::from_rgb(238, 232, 213),
                subtle_bg: Color32::from_rgb(220, 214, 195),
                text: Color32::from_rgb(101, 123, 131),
                muted: Color32::from_rgb(147, 161, 161),
                heading_colors: [
                    Color32::from_rgb(38, 139, 210), // Blue
                    Color32::from_rgb(42, 161, 152), // Cyan
                    Color32::from_rgb(133, 153, 0),  // Green
                    Color32::from_rgb(181, 137, 0),  // Yellow
                    Color32::from_rgb(203, 75, 22),   // Orange
                    Color32::from_rgb(211, 54, 130), // Magenta
                ],
                accent: Color32::from_rgb(38, 139, 210),
                link: Color32::from_rgb(38, 139, 210),
                code_bg: Color32::from_rgb(238, 232, 213),
                code_border: Color32::from_rgb(210, 204, 185),
                code_inline_bg: Color32::from_rgb(230, 224, 205),
                code_inline_text: Color32::from_rgb(181, 137, 0),
                quote_border: Color32::from_rgb(38, 139, 210),
                quote_bg: Color32::from_rgba_premultiplied(38, 139, 210, 20),
                table_header_bg: Color32::from_rgb(238, 232, 213),
                table_alt_bg: Color32::from_rgb(245, 239, 220),
                table_border: Color32::from_rgb(210, 204, 185),
                search_match_bg: Color32::from_rgba_premultiplied(181, 137, 0, 110),
                active_search_match_bg: Color32::from_rgba_premultiplied(203, 75, 22, 180),
                syntect_theme: "Solarized (light)",
            },
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThemePalette {
    pub bg: Color32,
    pub panel_bg: Color32,
    pub subtle_bg: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub heading_colors: [Color32; 6],
    pub accent: Color32,
    pub link: Color32,
    pub code_bg: Color32,
    pub code_border: Color32,
    pub code_inline_bg: Color32,
    pub code_inline_text: Color32,
    pub quote_border: Color32,
    pub quote_bg: Color32,
    pub table_header_bg: Color32,
    pub table_alt_bg: Color32,
    pub table_border: Color32,
    pub search_match_bg: Color32,
    pub active_search_match_bg: Color32,
    pub syntect_theme: &'static str,
}

impl ThemePalette {
    pub fn to_visuals(&self, is_dark: bool) -> Visuals {
        let mut visuals = if is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.panel_fill = self.panel_bg;
        visuals.window_fill = self.bg;
        visuals.extreme_bg_color = self.bg;

        visuals.widgets.noninteractive.bg_fill = self.panel_bg;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.code_border);

        visuals.widgets.inactive.bg_fill = self.subtle_bg;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.code_border);

        visuals.widgets.hovered.bg_fill = self.subtle_bg;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);

        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.bg);

        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke = Stroke::new(1.0, self.bg);

        visuals
    }

    pub fn alert_border(&self, kind: AlertKind) -> Color32 {
        let is_dark = (self.bg.r() as u32 + self.bg.g() as u32 + self.bg.b() as u32) < 380;
        if is_dark {
            match kind {
                AlertKind::Note => Color32::from_rgb(88, 166, 255),
                AlertKind::Tip => Color32::from_rgb(63, 185, 80),
                AlertKind::Important => Color32::from_rgb(163, 113, 247),
                AlertKind::Warning => Color32::from_rgb(210, 153, 34),
                AlertKind::Caution => Color32::from_rgb(248, 81, 73),
            }
        } else {
            match kind {
                AlertKind::Note => Color32::from_rgb(9, 105, 218),
                AlertKind::Tip => Color32::from_rgb(26, 127, 55),
                AlertKind::Important => Color32::from_rgb(130, 80, 223),
                AlertKind::Warning => Color32::from_rgb(154, 103, 0),
                AlertKind::Caution => Color32::from_rgb(207, 34, 46),
            }
        }
    }

    pub fn alert_bg(&self, kind: AlertKind) -> Color32 {
        let border = self.alert_border(kind);
        let is_dark = (self.bg.r() as u32 + self.bg.g() as u32 + self.bg.b() as u32) < 380;
        let alpha = if is_dark { 25 } else { 18 };
        Color32::from_rgba_premultiplied(border.r(), border.g(), border.b(), alpha)
    }

    pub fn alert_title_color(&self, kind: AlertKind) -> Color32 {
        self.alert_border(kind)
    }

    pub fn hex(c: Color32) -> String {
        format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
    }

    pub fn to_css(&self) -> String {
        format!(
            r#"
body {{
    background-color: {bg};
    color: {text};
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
    line-height: 1.6;
    padding: 2rem 4rem;
    max-width: 900px;
    margin: 0 auto;
}}
h1 {{ color: {h1}; border-bottom: 1px solid {border}; padding-bottom: 0.3em; }}
h2 {{ color: {h2}; border-bottom: 1px solid {border}; padding-bottom: 0.3em; }}
h3 {{ color: {h3}; }}
h4 {{ color: {h4}; }}
h5 {{ color: {h5}; }}
h6 {{ color: {h6}; }}
a {{ color: {link}; text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
code {{
    background-color: {code_inline_bg};
    color: {code_inline_text};
    padding: 0.2em 0.4em;
    border-radius: 4px;
    font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
    font-size: 85%;
}}
pre {{
    background-color: {code_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
    line-height: 1.45;
}}
pre code {{
    background-color: transparent;
    color: inherit;
    padding: 0;
}}
blockquote {{
    border-left: 4px solid {quote_border};
    background-color: {quote_bg};
    margin: 0;
    padding: 0.5em 1em;
    color: {muted};
}}
table {{
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}}
th, td {{
    border: 1px solid {table_border};
    padding: 8px 12px;
}}
th {{
    background-color: {table_header_bg};
    font-weight: 600;
}}
tr:nth-child(even) {{
    background-color: {table_alt_bg};
}}
hr {{
    border: none;
    border-top: 1px solid {border};
    margin: 2em 0;
}}
.markdown-alert {{
    border-left: 4px solid;
    padding: 0.5rem 1rem;
    margin: 1rem 0;
    border-radius: 4px;
}}
.markdown-alert-title {{
    font-weight: 600;
    margin-top: 0;
    margin-bottom: 0.4rem;
}}
.markdown-alert-note {{ border-color: #2f81f7; background-color: rgba(47, 129, 247, 0.1); }}
.markdown-alert-tip {{ border-color: #3fb950; background-color: rgba(63, 185, 80, 0.1); }}
.markdown-alert-important {{ border-color: #a371f7; background-color: rgba(163, 113, 247, 0.1); }}
.markdown-alert-warning {{ border-color: #d29922; background-color: rgba(210, 153, 34, 0.1); }}
.markdown-alert-caution {{ border-color: #f85149; background-color: rgba(248, 81, 73, 0.1); }}
"#,
            bg = Self::hex(self.bg),
            text = Self::hex(self.text),
            h1 = Self::hex(self.heading_colors[0]),
            h2 = Self::hex(self.heading_colors[1]),
            h3 = Self::hex(self.heading_colors[2]),
            h4 = Self::hex(self.heading_colors[3]),
            h5 = Self::hex(self.heading_colors[4]),
            h6 = Self::hex(self.heading_colors[5]),
            border = Self::hex(self.code_border),
            link = Self::hex(self.link),
            code_inline_bg = Self::hex(self.code_inline_bg),
            code_inline_text = Self::hex(self.code_inline_text),
            code_bg = Self::hex(self.code_bg),
            quote_border = Self::hex(self.quote_border),
            quote_bg = format!("rgba({}, {}, {}, 0.15)", self.quote_border.r(), self.quote_border.g(), self.quote_border.b()),
            muted = Self::hex(self.muted),
            table_border = Self::hex(self.table_border),
            table_header_bg = Self::hex(self.table_header_bg),
            table_alt_bg = Self::hex(self.table_alt_bg)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_have_valid_palettes() {
        for theme in ThemeKind::ALL {
            let p = theme.palette();
            assert_eq!(p.heading_colors.len(), 6);
            assert!(!p.syntect_theme.is_empty());
        }
    }

    #[test]
    fn test_theme_id_roundtrip() {
        for theme in ThemeKind::ALL {
            let id = theme.id();
            let parsed = ThemeKind::from_id(id);
            assert_eq!(*theme, parsed);
        }
    }
}
