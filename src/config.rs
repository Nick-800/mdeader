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
