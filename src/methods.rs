use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

/// Supported image extensions
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// Get a random wallpaper from the specified directory
pub fn get_random_wallpaper(dir: &PathBuf) -> Result<Option<PathBuf>> {
    if !dir.exists() {
        anyhow::bail!("Wallpaper directory does not exist: {}", dir.display());
    }

    let mut wallpapers: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if wallpapers.is_empty() {
        return Ok(None);
    }

    wallpapers.shuffle(&mut rand::thread_rng());
    Ok(Some(wallpapers.remove(0)))
}

/// Get the focused monitor name using jq or fallback to text parsing
pub fn get_monitor() -> Result<String> {
    // Try using jq first
    if let Ok(output) = Command::new("hyprctl")
        .args(["-j", "monitors"])
        .output()
    {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(monitors) = json.as_array() {
                    // Try to get focused monitor
                    for monitor in monitors {
                        if monitor.get("focused").and_then(|f| f.as_bool()).unwrap_or(false) {
                            if let Some(name) = monitor.get("name").and_then(|n| n.as_str()) {
                                return Ok(name.to_string());
                            }
                        }
                    }
                    // Fallback to first monitor
                    if let Some(monitor) = monitors.first() {
                        if let Some(name) = monitor.get("name").and_then(|n| n.as_str()) {
                            return Ok(name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback to text parsing
    if let Ok(output) = Command::new("hyprctl")
        .arg("monitors")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Monitor ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }

    Ok("all".to_string())
}

/// Change wallpaper using hyprctl
pub fn change_wallpaper_hyprctl(
    wallpaper: &PathBuf,
    monitor: &str,
    unload_unused: bool,
) -> Result<()> {
    tracing::info!("Attempting to change wallpaper...");

    // Unload unused wallpapers if enabled
    if unload_unused {
        Command::new("hyprctl")
            .args(["hyprpaper", "unload", "all"])
            .output()
            .context("Failed to unload unused wallpapers")?;
        tracing::info!("✓ Unloaded unused wallpapers");
    }

    // Preload the wallpaper
    let preload_status = Command::new("hyprctl")
        .args(["hyprpaper", "preload", &wallpaper.to_string_lossy()])
        .output()
        .context("Failed to preload wallpaper")?;

    if preload_status.status.success() {
        tracing::info!("✓ Preloaded: {}", wallpaper.display());
    } else {
        anyhow::bail!("Failed to preload wallpaper");
    }

    // Set wallpaper
    let wallpaper_arg = format!("{},{}", monitor, wallpaper.to_string_lossy());
    let set_status = Command::new("hyprctl")
        .args(["hyprpaper", "wallpaper", &wallpaper_arg])
        .output()
        .context("Failed to set wallpaper")?;

    if set_status.status.success() {
        tracing::info!("✓ Changed wallpaper to: {}", wallpaper.display());
        Ok(())
    } else {
        anyhow::bail!("Failed to set wallpaper");
    }
}

/// Change wallpaper using socket communication
pub fn change_wallpaper_socket(
    wallpaper: &PathBuf,
    monitor: &str,
    socket_path: &str,
) -> Result<()> {
    tracing::info!("Attempting to change wallpaper...");

    // Preload
    let preload_cmd = format!("preload {}", wallpaper.to_string_lossy());
    let preload_output = Command::new("socat")
        .args(["-", &format!("UNIX-CONNECT:{}", socket_path)])
        .arg(preload_cmd)
        .output()
        .context("Failed to communicate with hyprpaper socket")?;

    if preload_output.status.success() {
        tracing::info!("✓ Preloaded: {}", wallpaper.display());
    } else {
        anyhow::bail!("Failed to preload wallpaper");
    }

    // Set wallpaper
    let set_cmd = format!("wallpaper {},{}", monitor, wallpaper.to_string_lossy());
    let set_output = Command::new("socat")
        .args(["-", &format!("UNIX-CONNECT:{}", socket_path)])
        .arg(set_cmd)
        .output()
        .context("Failed to communicate with hyprpaper socket")?;

    if set_output.status.success() {
        tracing::info!("✓ Changed wallpaper to: {}", wallpaper.display());
        Ok(())
    } else {
        anyhow::bail!("Failed to set wallpaper");
    }
}

/// Change wallpaper using wtype (Wayland-native)
pub fn change_wallpaper_wtype() -> Result<()> {
    // Simulate Ctrl+Logo+Space using wtype
    let output = Command::new("wtype")
        .args(["-M", "ctrl", "-M", "logo", "-k", "space", "-m", "logo", "-m", "ctrl"])
        .output()
        .context("Failed to execute wtype")?;

    if output.status.success() {
        tracing::info!("✓ Triggered wallpaper change via wtype");
        Ok(())
    } else {
        anyhow::bail!("Failed to trigger wallpaper change via wtype");
    }
}

/// Change wallpaper using ydotool
pub fn change_wallpaper_ydotool() -> Result<()> {
    // Simulate Ctrl+Super+Space using ydotool
    // Key codes: 29=Ctrl, 125=Super, 57=Space
    let output = Command::new("ydotool")
        .args(["key", "29:1", "125:1", "57:1", "57:0", "125:0", "29:0"])
        .output()
        .context("Failed to execute ydotool")?;

    if output.status.success() {
        tracing::info!("✓ Triggered wallpaper change via ydotool");
        Ok(())
    } else {
        anyhow::bail!("Failed to trigger wallpaper change via ydotool");
    }
}
