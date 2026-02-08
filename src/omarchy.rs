use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Get Omarchy's current theme background directory
pub fn get_omarchy_bg_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"));

    let omarchy_current = config_dir
        .join("omarchy")
        .join("current")
        .join("theme");

    let omarchy_bg = omarchy_current.join("backgrounds");

    if omarchy_bg.exists() {
        Ok(omarchy_bg)
    } else {
        anyhow::bail!(
            "Omarchy backgrounds directory not found: {}",
            omarchy_bg.display()
        )
    }
}

/// Check if Omarchy is available
pub fn is_omarchy_available() -> bool {
    Command::new("omarchy-theme-bg-next")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Change wallpaper using Omarchy's command
pub fn change_wallpaper_omarchy() -> Result<()> {
    let output = Command::new("omarchy-theme-bg-next")
        .output()
        .context("Failed to execute omarchy-theme-bg-next")?;

    if output.status.success() {
        tracing::info!("✓ Changed wallpaper via Omarchy");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to change wallpaper via Omarchy: {}", stderr);
    }
}
