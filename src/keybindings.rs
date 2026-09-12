use egui::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    #[serde(default = "default_open_file")]
    pub open_file: String,

    #[serde(default = "default_reload")]
    pub reload: String,

    #[serde(default = "default_find")]
    pub find: String,

    #[serde(default = "default_toggle_toc")]
    pub toggle_toc: String,

    #[serde(default = "default_toggle_ai")]
    pub toggle_ai: String,

    #[serde(default = "default_toggle_zen")]
    pub toggle_zen: String,

    #[serde(default = "default_toggle_slides")]
    pub toggle_slides: String,

    #[serde(default = "default_export_html")]
    pub export_html: String,

    #[serde(default = "default_zoom_in")]
    pub zoom_in: String,

    #[serde(default = "default_zoom_out")]
    pub zoom_out: String,

    #[serde(default = "default_zoom_reset")]
    pub zoom_reset: String,

    #[serde(default = "default_scroll_top")]
    pub scroll_top: String,

    #[serde(default = "default_scroll_bottom")]
    pub scroll_bottom: String,

    #[serde(default = "default_escape")]
    pub escape: String,

    #[serde(default)]
    pub vim: VimKeybindingsConfig,
}

fn default_open_file() -> String {
    "Ctrl+O".to_string()
}
fn default_reload() -> String {
    "Ctrl+R".to_string()
}
fn default_find() -> String {
    "Ctrl+F".to_string()
}
fn default_toggle_toc() -> String {
    "Ctrl+B".to_string()
}
fn default_toggle_ai() -> String {
    "Ctrl+Shift+A".to_string()
}
fn default_toggle_zen() -> String {
    "F11".to_string()
}
fn default_toggle_slides() -> String {
    "F5".to_string()
}
fn default_export_html() -> String {
    "Ctrl+E".to_string()
}
fn default_zoom_in() -> String {
    "Ctrl+Plus".to_string()
}
fn default_zoom_out() -> String {
    "Ctrl+Minus".to_string()
}
fn default_zoom_reset() -> String {
    "Ctrl+0".to_string()
}
fn default_scroll_top() -> String {
    "Home".to_string()
}
fn default_scroll_bottom() -> String {
    "End".to_string()
}
fn default_escape() -> String {
    "Escape".to_string()
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            open_file: default_open_file(),
            reload: default_reload(),
            find: default_find(),
            toggle_toc: default_toggle_toc(),
            toggle_ai: default_toggle_ai(),
            toggle_zen: default_toggle_zen(),
            toggle_slides: default_toggle_slides(),
            export_html: default_export_html(),
            zoom_in: default_zoom_in(),
            zoom_out: default_zoom_out(),
            zoom_reset: default_zoom_reset(),
            scroll_top: default_scroll_top(),
            scroll_bottom: default_scroll_bottom(),
            escape: default_escape(),
            vim: VimKeybindingsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimKeybindingsConfig {
    #[serde(default = "default_vim_scroll_down")]
    pub scroll_down: String,

    #[serde(default = "default_vim_scroll_up")]
    pub scroll_up: String,

    #[serde(default = "default_vim_scroll_down_half")]
    pub scroll_down_half: String,

    #[serde(default = "default_vim_scroll_up_half")]
    pub scroll_up_half: String,

    #[serde(default = "default_vim_scroll_top")]
    pub scroll_top: String,

    #[serde(default = "default_vim_scroll_bottom")]
    pub scroll_bottom: String,

    #[serde(default = "default_vim_find")]
    pub find: String,

    #[serde(default = "default_vim_next_match")]
    pub next_match: String,

    #[serde(default = "default_vim_prev_match")]
    pub prev_match: String,

    #[serde(default = "default_vim_toggle_toc")]
    pub toggle_toc: String,

    #[serde(default = "default_vim_toggle_ai")]
    pub toggle_ai: String,

    #[serde(default = "default_vim_toggle_zen")]
    pub toggle_zen: String,

    #[serde(default = "default_vim_toggle_slides")]
    pub toggle_slides: String,

    #[serde(default = "default_vim_reload")]
    pub reload: String,

    #[serde(default = "default_vim_open_file")]
    pub open_file: String,

    #[serde(default = "default_vim_copy_raw")]
    pub copy_raw: String,
}

fn default_vim_scroll_down() -> String {
    "j".to_string()
}
fn default_vim_scroll_up() -> String {
    "k".to_string()
}
fn default_vim_scroll_down_half() -> String {
    "Ctrl+d".to_string()
}
fn default_vim_scroll_up_half() -> String {
    "Ctrl+u".to_string()
}
fn default_vim_scroll_top() -> String {
    "gg".to_string()
}
fn default_vim_scroll_bottom() -> String {
    "G".to_string()
}
fn default_vim_find() -> String {
    "/".to_string()
}
fn default_vim_next_match() -> String {
    "n".to_string()
}
fn default_vim_prev_match() -> String {
    "N".to_string()
}
fn default_vim_toggle_toc() -> String {
    "b".to_string()
}
fn default_vim_toggle_ai() -> String {
    "a".to_string()
}
fn default_vim_toggle_zen() -> String {
    "z".to_string()
}
fn default_vim_toggle_slides() -> String {
    "p".to_string()
}
fn default_vim_reload() -> String {
    "r".to_string()
}
fn default_vim_open_file() -> String {
    "o".to_string()
}
fn default_vim_copy_raw() -> String {
    "yy".to_string()
}

impl Default for VimKeybindingsConfig {
    fn default() -> Self {
        Self {
            scroll_down: default_vim_scroll_down(),
            scroll_up: default_vim_scroll_up(),
            scroll_down_half: default_vim_scroll_down_half(),
            scroll_up_half: default_vim_scroll_up_half(),
            scroll_top: default_vim_scroll_top(),
            scroll_bottom: default_vim_scroll_bottom(),
            find: default_vim_find(),
            next_match: default_vim_next_match(),
            prev_match: default_vim_prev_match(),
            toggle_toc: default_vim_toggle_toc(),
            toggle_ai: default_vim_toggle_ai(),
            toggle_zen: default_vim_toggle_zen(),
            toggle_slides: default_vim_toggle_slides(),
            reload: default_vim_reload(),
            open_file: default_vim_open_file(),
            copy_raw: default_vim_copy_raw(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedShortcut {
    Single {
        ctrl: bool,
        shift: bool,
        alt: bool,
        key: Key,
    },
    Chord(String),
}

impl ParsedShortcut {
    pub fn parse(s: &str) -> Option<Self> {
        let raw = s.trim();
        if raw.is_empty() {
            return None;
        }

        // Two identical letters like "gg" or "yy"
        if raw.len() == 2 && raw.chars().nth(0) == raw.chars().nth(1) && raw.chars().all(|c| c.is_alphabetic()) {
            return Some(ParsedShortcut::Chord(raw.to_lowercase()));
        }

        // Single punctuation characters without modifiers
        if raw == "+" {
            return Some(ParsedShortcut::Single {
                ctrl: false,
                shift: false,
                alt: false,
                key: Key::Plus,
            });
        }
        if raw == "=" {
            return Some(ParsedShortcut::Single {
                ctrl: false,
                shift: false,
                alt: false,
                key: Key::Equals,
            });
        }
        if raw == "-" {
            return Some(ParsedShortcut::Single {
                ctrl: false,
                shift: false,
                alt: false,
                key: Key::Minus,
            });
        }
        if raw == "/" {
            return Some(ParsedShortcut::Single {
                ctrl: false,
                shift: false,
                alt: false,
                key: Key::Slash,
            });
        }

        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;

        // Split on '+' but handle trailing '+' like "Ctrl++"
        let mut parts = Vec::new();
        let mut remainder = raw;

        if remainder.ends_with("++") {
            remainder = &remainder[..remainder.len() - 1];
            parts.extend(remainder.split('+').filter(|p| !p.is_empty()));
            parts.push("+");
        } else if remainder.ends_with("+=") {
            remainder = &remainder[..remainder.len() - 2];
            parts.extend(remainder.split('+').filter(|p| !p.is_empty()));
            parts.push("=");
        } else if remainder.ends_with("+-") {
            remainder = &remainder[..remainder.len() - 2];
            parts.extend(remainder.split('+').filter(|p| !p.is_empty()));
            parts.push("-");
        } else {
            parts.extend(remainder.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()));
        }

        if parts.is_empty() {
            return None;
        }

        let key_str = parts.pop()?;
        for modifier in parts {
            match modifier.to_lowercase().as_str() {
                "ctrl" | "control" | "cmd" | "command" => ctrl = true,
                "shift" => shift = true,
                "alt" | "opt" | "option" => alt = true,
                _ => {}
            }
        }

        // Parse key
        let key = parse_key(key_str);
        if let Some(k) = key {
            // If single uppercase letter without explicit shift (e.g. "G" or "N"), auto-add shift
            if key_str.len() == 1 && key_str.chars().next().unwrap().is_ascii_uppercase() && !ctrl && !alt {
                shift = true;
            }
            Some(ParsedShortcut::Single {
                ctrl,
                shift,
                alt,
                key: k,
            })
        } else {
            None
        }
    }

    pub fn matches(&self, event_key: Key, modifiers: &Modifiers) -> bool {
        match self {
            ParsedShortcut::Single { ctrl, shift, alt, key } => {
                let has_ctrl = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;
                if *ctrl != has_ctrl {
                    return false;
                }
                if *alt != modifiers.alt {
                    return false;
                }

                // Smart matching for Plus / Equals
                if *key == Key::Plus || *key == Key::Equals {
                    if event_key == Key::Plus || event_key == Key::Equals {
                        return true;
                    }
                }

                if *shift != modifiers.shift {
                    // If event was Plus and shift was pressed to type it, allow it
                    if event_key == Key::Plus || event_key == Key::Equals {
                        // ignore shift variation
                    } else {
                        return false;
                    }
                }

                *key == event_key
            }
            ParsedShortcut::Chord(_) => false,
        }
    }
}

fn parse_key(name: &str) -> Option<Key> {
    if let Some(k) = Key::from_name(name) {
        return Some(k);
    }

    match name.to_lowercase().as_str() {
        "esc" | "escape" => Some(Key::Escape),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        "enter" | "return" => Some(Key::Enter),
        "space" => Some(Key::Space),
        "tab" => Some(Key::Tab),
        "backspace" => Some(Key::Backspace),
        "up" | "arrowup" => Some(Key::ArrowUp),
        "down" | "arrowdown" => Some(Key::ArrowDown),
        "left" | "arrowleft" => Some(Key::ArrowLeft),
        "right" | "arrowright" => Some(Key::ArrowRight),
        "+" | "plus" => Some(Key::Plus),
        "-" | "minus" => Some(Key::Minus),
        "=" | "equals" | "equal" => Some(Key::Equals),
        "/" | "slash" => Some(Key::Slash),
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),
        _ => None,
    }
}

pub struct KeyActionManager {
    pub pending_chord: String,
    pub last_chord_time: Option<Instant>,
}

impl Default for KeyActionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyActionManager {
    pub fn new() -> Self {
        Self {
            pending_chord: String::new(),
            last_chord_time: None,
        }
    }

    pub fn record_key(&mut self, key_str: &str) -> Option<String> {
        if let Some(t) = self.last_chord_time {
            if t.elapsed() > Duration::from_millis(1000) {
                self.pending_chord.clear();
                self.last_chord_time = None;
            }
        }

        if self.pending_chord.is_empty() {
            self.pending_chord = key_str.to_string();
            self.last_chord_time = Some(Instant::now());
            None
        } else {
            let combined = format!("{}{}", self.pending_chord, key_str);
            self.pending_chord.clear();
            self.last_chord_time = None;
            Some(combined)
        }
    }

    pub fn clear(&mut self) {
        self.pending_chord.clear();
        self.last_chord_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcuts() {
        let sc = ParsedShortcut::parse("Ctrl+O").unwrap();
        assert_eq!(
            sc,
            ParsedShortcut::Single {
                ctrl: true,
                shift: false,
                alt: false,
                key: Key::O,
            }
        );

        let sc2 = ParsedShortcut::parse("Ctrl+Shift+A").unwrap();
        assert_eq!(
            sc2,
            ParsedShortcut::Single {
                ctrl: true,
                shift: true,
                alt: false,
                key: Key::A,
            }
        );

        let sc3 = ParsedShortcut::parse("F11").unwrap();
        assert_eq!(
            sc3,
            ParsedShortcut::Single {
                ctrl: false,
                shift: false,
                alt: false,
                key: Key::F11,
            }
        );

        let sc4 = ParsedShortcut::parse("gg").unwrap();
        assert_eq!(sc4, ParsedShortcut::Chord("gg".to_string()));

        let sc5 = ParsedShortcut::parse("G").unwrap();
        assert_eq!(
            sc5,
            ParsedShortcut::Single {
                ctrl: false,
                shift: true,
                alt: false,
                key: Key::G,
            }
        );

        let sc6 = ParsedShortcut::parse("Ctrl++").unwrap();
        assert!(matches!(sc6, ParsedShortcut::Single { ctrl: true, key: Key::Plus, .. }));

        let sc7 = ParsedShortcut::parse("Ctrl+-").unwrap();
        assert!(matches!(sc7, ParsedShortcut::Single { ctrl: true, key: Key::Minus, .. }));
    }

    #[test]
    fn test_shortcut_matching() {
        let sc = ParsedShortcut::parse("Ctrl+O").unwrap();
        let mut mods = Modifiers::default();
        mods.ctrl = true;
        assert!(sc.matches(Key::O, &mods));
        assert!(!sc.matches(Key::P, &mods));

        mods.ctrl = false;
        assert!(!sc.matches(Key::O, &mods));
    }
}
