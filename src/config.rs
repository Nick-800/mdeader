use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_font_size")]
    pub font_size: f32,

    #[serde(default = "default_line_spacing")]
    pub line_spacing: f32,

    #[serde(default = "default_show_toc")]
    pub show_toc: bool,

    #[serde(default = "default_toc_width")]
    pub toc_width: f32,

    #[serde(default)]
    pub recent_files: Vec<PathBuf>,

    #[serde(default = "default_watch_mode")]
    pub watch_mode: bool,

    #[serde(default = "default_zoom")]
    pub zoom: f32,

    #[serde(default = "default_show_ai")]
    pub show_ai: bool,

    #[serde(default = "default_ai_width")]
    pub ai_width: f32,

    #[serde(default)]
    pub ai: AiConfig,

    #[serde(default = "default_ipc_enabled")]
    pub ipc_enabled: bool,

    #[serde(default = "default_ipc_port")]
    pub ipc_port: u16,

    #[serde(default = "default_vim_mode")]
    pub vim_mode: bool,

    #[serde(default = "default_max_content_width")]
    pub max_content_width: f32,

    #[serde(default)]
    pub keybindings: crate::keybindings::KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_provider")]
    pub provider: String,

    #[serde(default = "default_ai_endpoint")]
    pub endpoint: String,

    #[serde(default = "default_ai_model")]
    pub model: String,

    #[serde(default)]
    pub api_key: Option<String>,
}

fn default_ai_provider() -> String {
    "ollama".to_string()
}

fn default_ai_endpoint() -> String {
    "http://localhost:11434".to_string()
}

fn default_ai_model() -> String {
    "llama3.2".to_string()
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            endpoint: default_ai_endpoint(),
            model: default_ai_model(),
            api_key: None,
        }
    }
}

fn default_show_ai() -> bool {
    false
}

fn default_ai_width() -> f32 {
    340.0
}

fn default_ipc_enabled() -> bool {
    true
}

fn default_ipc_port() -> u16 {
    19842
}

fn default_vim_mode() -> bool {
    false
}

fn default_max_content_width() -> f32 {
    860.0
}

fn default_theme() -> String {
    "GitHubDark".to_string()
}

fn default_font_size() -> f32 {
    15.0
}

fn default_line_spacing() -> f32 {
    1.4
}

fn default_show_toc() -> bool {
    true
}

fn default_toc_width() -> f32 {
    240.0
}

fn default_watch_mode() -> bool {
    true
}

fn default_zoom() -> f32 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_size: default_font_size(),
            line_spacing: default_line_spacing(),
            show_toc: default_show_toc(),
            toc_width: default_toc_width(),
            recent_files: Vec::new(),
            watch_mode: default_watch_mode(),
            zoom: default_zoom(),
            show_ai: default_show_ai(),
            ai_width: default_ai_width(),
            ai: AiConfig::default(),
            ipc_enabled: default_ipc_enabled(),
            ipc_port: default_ipc_port(),
            vim_mode: default_vim_mode(),
            max_content_width: default_max_content_width(),
            keybindings: crate::keybindings::KeybindingsConfig::default(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "mdeader", "mdeader").map(|dirs| {
            let config_dir = dirs.config_dir();
            config_dir.join("config.toml")
        })
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        if !path.exists() {
            let default_cfg = Self::default();
            let _ = default_cfg.save();
            return default_cfg;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let toml_str = toml::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(path, toml_str)?;
        }
        Ok(())
    }

    pub fn add_recent_file(&mut self, path: &Path) {
        let path_buf = path.to_path_buf();
        self.recent_files.retain(|p| p != &path_buf);
        self.recent_files.insert(0, path_buf);
        if self.recent_files.len() > 15 {
            self.recent_files.truncate(15);
        }
        let _ = self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "GitHubDark");
        assert_eq!(cfg.font_size, 15.0);
        assert!(cfg.show_toc);
        assert!(cfg.watch_mode);
        assert_eq!(cfg.zoom, 1.0);
        assert!(!cfg.show_ai);
        assert_eq!(cfg.ai.provider, "ollama");
        assert_eq!(cfg.ai.endpoint, "http://localhost:11434");
        assert_eq!(cfg.ai.model, "llama3.2");
        assert!(cfg.ipc_enabled);
        assert_eq!(cfg.ipc_port, 19842);
    }

    #[test]
    fn test_recent_files() {
        let mut cfg = Config::default();
        let p1 = PathBuf::from("/tmp/test1.md");
        let p2 = PathBuf::from("/tmp/test2.md");
        cfg.add_recent_file(&p1);
        cfg.add_recent_file(&p2);
        assert_eq!(cfg.recent_files[0], p2);
        assert_eq!(cfg.recent_files[1], p1);
        cfg.add_recent_file(&p1);
        assert_eq!(cfg.recent_files[0], p1);
        assert_eq!(cfg.recent_files[1], p2);
    }
}
