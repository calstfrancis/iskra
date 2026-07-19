use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Root of the Iskra data folder (and, once backup is set up, the git
    /// repo). Sermons live in the `sermons/` subdirectory — see `sermons_dir`.
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_true")]
    pub sidebar_visible: bool,
    #[serde(default = "default_sidebar_width_fraction")]
    pub sidebar_width_fraction: f64,
    #[serde(default = "default_window_width")]
    pub window_width: i32,
    #[serde(default = "default_window_height")]
    pub window_height: i32,
    #[serde(default)]
    pub window_maximized: bool,
    #[serde(default)]
    pub last_sermon: Option<PathBuf>,
    #[serde(default)]
    pub recent_sermons: Vec<PathBuf>,
    #[serde(default = "default_autosave_debounce_ms")]
    pub autosave_debounce_ms: u64,
    #[serde(default = "default_true")]
    pub print_include_notes: bool,
    #[serde(default)]
    pub print_include_tags: bool,
    #[serde(default = "default_print_font_pt")]
    pub print_font_pt: f64,
}

fn default_work_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde("~/Documents/Iskra").into_owned())
}

fn default_true() -> bool {
    true
}
fn default_sidebar_width_fraction() -> f64 {
    0.22
}
fn default_window_width() -> i32 {
    1200
}
fn default_window_height() -> i32 {
    800
}
fn default_autosave_debounce_ms() -> u64 {
    800
}
fn default_print_font_pt() -> f64 {
    13.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_dir: default_work_dir(),
            theme: Theme::default(),
            sidebar_visible: true,
            sidebar_width_fraction: default_sidebar_width_fraction(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            window_maximized: false,
            last_sermon: None,
            recent_sermons: Vec::new(),
            autosave_debounce_ms: default_autosave_debounce_ms(),
            print_include_notes: true,
            print_include_tags: false,
            print_font_pt: default_print_font_pt(),
        }
    }
}

impl Config {
    pub fn sermons_dir(&self) -> PathBuf {
        self.work_dir.join("sermons")
    }

    pub fn push_recent(&mut self, path: PathBuf) {
        self.recent_sermons.retain(|p| p != &path);
        self.recent_sermons.insert(0, path);
        if self.recent_sermons.len() > 10 {
            self.recent_sermons.truncate(10);
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_file();
        let text = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_file();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        crate::error::atomic_write(&path, toml::to_string(self)?.as_bytes())?;
        Ok(())
    }

    fn config_file() -> PathBuf {
        PathBuf::from(shellexpand::tilde("~/.config/iskra").into_owned()).join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_gets_all_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.work_dir, default_work_dir());
        assert!(cfg.sidebar_visible);
        assert_eq!(cfg.autosave_debounce_ms, 800);
    }

    #[test]
    fn unknown_fields_are_tolerated() {
        let cfg: Config = toml::from_str("future_field = true\nwindow_width = 900").unwrap();
        assert_eq!(cfg.window_width, 900);
    }

    #[test]
    fn sermons_dir_is_subdir_of_work_dir() {
        let cfg = Config::default();
        assert!(cfg.sermons_dir().ends_with("Iskra/sermons"));
    }
}
