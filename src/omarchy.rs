use anyhow::{Context, Result};
use std::process::Command;

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
