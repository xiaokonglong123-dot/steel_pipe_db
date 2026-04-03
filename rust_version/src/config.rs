
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub ui: UiConfig,
    pub colors: ColorsConfig,
    pub fonts: FontsConfig,
    pub language: LanguageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub icon_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    pub nav_bg: String,
    pub content_bg: String,
    pub button_bg: String,
    pub entry_btn: String,
    pub exit_btn: String,
    pub inventory_btn: String,
    pub records_btn: String,
    pub stats_btn: String,
    pub close_btn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontsConfig {
    pub font_family: String,
    pub title_size: u32,
    pub button_size: u32,
    pub label_size: u32,
    pub entry_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: "pipes.db".to_string(),
            },
            ui: UiConfig {
                window_title: "钢管原料进出入库管理系统".to_string(),
                window_width: 1200,
                window_height: 800,
                icon_path: "steel_pipe.ico".to_string(),
            },
            colors: ColorsConfig {
                nav_bg: "#2c3e50".to_string(),
                content_bg: "#ecf0f1".to_string(),
                button_bg: "#34495e".to_string(),
                entry_btn: "#3498db".to_string(),
                exit_btn: "#e74c3c".to_string(),
                inventory_btn: "#2ecc71".to_string(),
                records_btn: "#f39c12".to_string(),
                stats_btn: "#9b59b6".to_string(),
                close_btn: "#95a5a6".to_string(),
            },
            fonts: FontsConfig {
                font_family: "微软雅黑".to_string(),
                title_size: 18,
                button_size: 12,
                label_size: 10,
                entry_size: 10,
            },
            language: LanguageConfig {
                language: "zh_CN".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
