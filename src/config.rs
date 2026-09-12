use crate::theme::themes::ThemeId;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TabSessionInfo {
    pub file_path: Option<PathBuf>,
    pub file_name: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PaneSessionInfo {
    pub tabs: Vec<TabSessionInfo>,
    pub active_tab_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionConfig {
    pub root_dir: Option<PathBuf>,
    pub left_pane: PaneSessionInfo,
    pub right_pane: Option<PaneSessionInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarkdownSpec {
    #[default]
    #[serde(rename = "GFM", alias = "gfm")]
    Gfm,
    #[serde(rename = "CommonMark", alias = "common_mark", alias = "commonmark")]
    CommonMark,
}

impl MarkdownSpec {
    pub const ALL: &'static [MarkdownSpec] = &[MarkdownSpec::Gfm, MarkdownSpec::CommonMark];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gfm => "GFM (GitHub Flavored)",
            Self::CommonMark => "CommonMark (Standard)",
        }
    }
}

fn default_opacity_val() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeId,
    pub font: String,
    pub font_size: f32,
    pub opacity: f32,
    #[serde(default = "default_opacity_val")]
    pub file_tree_opacity: f32,
    #[serde(default = "default_opacity_val")]
    pub title_bar_opacity: f32,
    pub dimming: f32,
    pub split_layout: String,
    pub file_tree_visible: bool,
    pub ai_enabled: bool,
    pub ai_model: String,
    #[serde(default)]
    pub ai_chat_visible: bool,
    #[serde(default)]
    pub markdown_spec: MarkdownSpec,
    #[serde(default)]
    pub session: SessionConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeId::TokyoNight,
            font: "JetBrainsMono Nerd Font".to_string(),
            font_size: 14.0,
            opacity: 1.0,
            file_tree_opacity: 1.0,
            title_bar_opacity: 1.0,
            dimming: 0.0,
            split_layout: "Split".to_string(),
            file_tree_visible: true,
            ai_enabled: true,
            ai_model: "deepseek-coder-v2:16b".to_string(),
            ai_chat_visible: false,
            markdown_spec: MarkdownSpec::Gfm,
            session: SessionConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("org", "pop_os", "rooney") {
            proj_dirs.config_dir().join("config.toml")
        } else if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.config_dir().join("rooney").join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::editor::EditorTab::atomic_write_file(&path, &content)?;
        Ok(())
    }
}
