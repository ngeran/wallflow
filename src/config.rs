use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration file path
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("wallflow")
        .join("config")
}

/// Wallpaper rotation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperMethod {
    #[default]
    Omarchy,
    Hyprctl,
    Socket,
    Wtype,
    Ydotool,
}

impl WallpaperMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            WallpaperMethod::Omarchy => "omarchy",
            WallpaperMethod::Hyprctl => "hyprctl",
            WallpaperMethod::Socket => "socket",
            WallpaperMethod::Wtype => "wtype",
            WallpaperMethod::Ydotool => "ydotool",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "omarchy" => Ok(WallpaperMethod::Omarchy),
            "hyprctl" => Ok(WallpaperMethod::Hyprctl),
            "socket" => Ok(WallpaperMethod::Socket),
            "wtype" => Ok(WallpaperMethod::Wtype),
            "ydotool" => Ok(WallpaperMethod::Ydotool),
            _ => anyhow::bail!("Unknown wallpaper method: {}", s),
        }
    }
}

/// Wallpaper directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperDir {
    Path(PathBuf),
    Omarchy,
}

impl Default for WallpaperDir {
    fn default() -> Self {
        WallpaperDir::Omarchy
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Rotation interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u64,

    /// Wallpaper rotation method
    #[serde(default)]
    pub method: WallpaperMethod,

    /// Wallpaper directory source
    #[serde(default)]
    pub wallpaper_dir: WallpaperDir,

    /// Unload unused wallpapers to save RAM
    #[serde(default = "default_unload_unused")]
    pub unload_unused: bool,

    /// Log countdown timer
    #[serde(default = "default_log_countdown")]
    pub log_countdown: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval: default_interval(),
            method: WallpaperMethod::default(),
            wallpaper_dir: WallpaperDir::default(),
            unload_unused: default_unload_unused(),
            log_countdown: default_log_countdown(),
        }
    }
}

fn default_interval() -> u64 {
    180
}

fn default_unload_unused() -> bool {
    true
}

fn default_log_countdown() -> bool {
    true
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = config_path();

        if !path.exists() {
            // Create default config
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = config_path();

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        Ok(())
    }

    /// Get the actual wallpaper directory path
    pub fn get_wallpaper_dir(&self) -> Result<PathBuf> {
        match &self.wallpaper_dir {
            WallpaperDir::Path(path) => Ok(path.clone()),
            WallpaperDir::Omarchy => {
                // Use Omarchy's current theme backgrounds directory
                let omarchy_bg = dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.config"))
                    .join("omarchy")
                    .join("current")
                    .join("theme")
                    .join("backgrounds");

                if omarchy_bg.exists() {
                    Ok(omarchy_bg)
                } else {
                    // Fallback to current theme directory
                    let omarchy_current = dirs::config_dir()
                        .unwrap_or_else(|| PathBuf::from("~/.config"))
                        .join("omarchy")
                        .join("current")
                        .join("theme");

                    if omarchy_current.exists() {
                        Ok(omarchy_current)
                    } else {
                        anyhow::bail!("Omarchy directory not found: {}", omarchy_bg.display());
                    }
                }
            }
        }
    }
}
