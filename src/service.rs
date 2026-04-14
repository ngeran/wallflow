use anyhow::{Context, Result};
use chrono::{Local, Duration};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::time::interval;
use tracing::{error, info};

use crate::config::{Config, WallpaperDir, WallpaperMethod};
use crate::methods::*;
use crate::omarchy;

/// WallFlow service
pub struct WallFlowService {
    config: Config,
    running: Arc<AtomicBool>,
}

impl WallFlowService {
    /// Create a new service instance
    pub fn new(config: Config) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Run the service
    pub async fn run(&self) -> Result<()> {
        self.print_startup_info();

        // Validate wallpaper directory if needed
        self.validate_wallpaper_dir()?;

        // Get monitor name for methods that need it
        let monitor = if matches!(
            self.config.method,
            WallpaperMethod::Hyprctl | WallpaperMethod::Socket
        ) {
            Some(get_monitor()?)
        } else {
            None
        };

        // Change wallpaper immediately on start
        info!("Setting initial wallpaper...");
        self.change_wallpaper(monitor.as_deref()).await?;

        // Main rotation loop
        let interval_secs = self.config.interval;
        let mut timer = interval(StdDuration::from_secs(interval_secs));
        timer.tick().await; // Skip first immediate tick

        info!("Starting rotation loop...");

        while self.running.load(Ordering::Relaxed) {
            // Calculate and log next change time
            let next_change = Local::now() + Duration::seconds(interval_secs as i64);
            info!("Next wallpaper change at: {}", next_change.format("%Y-%m-%d %H:%M:%S"));

            // Countdown if enabled
            if self.config.log_countdown {
                self.countdown(interval_secs).await;
            } else {
                timer.tick().await;
            }

            // Check if we should still be running
            if !self.running.load(Ordering::Relaxed) {
                break;
            }

            // Change wallpaper
            if let Err(e) = self.change_wallpaper(monitor.as_deref()).await {
                error!("Failed to change wallpaper: {}", e);
            }
        }

        info!("WallFlow service stopped");
        Ok(())
    }

    /// Print startup information
    fn print_startup_info(&self) {
        info!("════════════════════════════════════════════════════════════════");
        info!("🌊 WallFlow Started");
        info!("════════════════════════════════════════════════════════════════");
        info!("Configuration:");
        info!("  Method:           {}", self.config.method.as_str());
        match &self.config.wallpaper_dir {
            WallpaperDir::Omarchy => {
                info!("  Source:           Omarchy (dynamic theme detection)");
            }
            WallpaperDir::Path(path) => {
                info!("  Wallpaper Dir:    {}", path.display());
            }
        }
        info!("  Interval:         {} seconds ({})", self.config.interval, Self::format_time(self.config.interval));
        info!("  Log Countdown:    {}", self.config.log_countdown);
        info!("  Unload Unused:    {}", self.config.unload_unused);

        match self.config.method {
            WallpaperMethod::Omarchy => {
                info!("  Using:            Omarchy bg-next command");
            }
            WallpaperMethod::Hyprctl => {
                info!("  Using:            hyprctl commands directly");
            }
            WallpaperMethod::Socket => {
                info!("  Socket Method:    Direct socket communication");
            }
            WallpaperMethod::Wtype => {
                info!("  Simulating:       Ctrl+Super+Space via wtype");
            }
            WallpaperMethod::Ydotool => {
                info!("  Simulating:       Ctrl+Super+Space via ydotool");
            }
        }
        info!("════════════════════════════════════════════════════════════════");
    }

    /// Validate wallpaper directory
    fn validate_wallpaper_dir(&self) -> Result<()> {
        // Skip for Omarchy method (it handles everything)
        if self.config.method == WallpaperMethod::Omarchy {
            return Ok(());
        }

        let wp_dir = self.config.get_wallpaper_dir()
            .context("Failed to get wallpaper directory")?;

        if !wp_dir.exists() {
            anyhow::bail!("Wallpaper directory does not exist: {}", wp_dir.display());
        }

        // Count wallpapers
        let count = self.count_wallpapers(&wp_dir)?;
        info!("Found {} wallpaper(s) in directory", count);

        if count == 0 {
            anyhow::bail!("No wallpapers found in {}", wp_dir.display());
        }

        Ok(())
    }

    /// Count wallpapers in directory
    fn count_wallpapers(&self, dir: &PathBuf) -> Result<usize> {
        Ok(walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        matches!(
                            ext.to_lowercase().as_str(),
                            "jpg" | "jpeg" | "png" | "webp"
                        )
                    })
                    .unwrap_or(false)
            })
            .count())
    }

    /// Format duration as human-readable string
    fn format_time(seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Countdown timer with logging
    async fn countdown(&self, total_seconds: u64) {
        let mut remaining = total_seconds;

        while remaining > 0 && self.running.load(Ordering::Relaxed) {
            // Log at specific intervals
            if remaining == total_seconds
                || remaining == 60
                || remaining == 30
                || remaining == 10
                || remaining <= 5
            {
                info!(
                    "Time until next change: {}",
                    Self::format_time(remaining)
                );
            }

            tokio::time::sleep(StdDuration::from_secs(1)).await;
            remaining -= 1;
        }
    }

    /// Change wallpaper using the configured method
    async fn change_wallpaper(&self, monitor: Option<&str>) -> Result<()> {
        match self.config.method {
            WallpaperMethod::Omarchy => {
                omarchy::change_wallpaper_omarchy()?;
            }
            WallpaperMethod::Hyprctl => {
                let wp_dir = self.config.get_wallpaper_dir()?;
                if let Some(wallpaper) = get_random_wallpaper(&wp_dir)? {
                    change_wallpaper_hyprctl(
                        &wallpaper,
                        monitor.unwrap_or("all"),
                        self.config.unload_unused,
                    )?;
                } else {
                    anyhow::bail!("No wallpapers found in {}", wp_dir.display());
                }
            }
            WallpaperMethod::Socket => {
                let wp_dir = self.config.get_wallpaper_dir()?;
                if let Some(wallpaper) = get_random_wallpaper(&wp_dir)? {
                    // Get socket path from environment
                    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
                        .unwrap_or_default();
                    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
                        .unwrap_or_else(|_| "/tmp".to_string());
                    let socket_path = format!("{}/hypr/{}/.hyprpaper.sock", xdg_runtime, sig);

                    change_wallpaper_socket(&wallpaper, monitor.unwrap_or("all"), &socket_path)?;
                } else {
                    anyhow::bail!("No wallpapers found in {}", wp_dir.display());
                }
            }
            WallpaperMethod::Wtype => {
                change_wallpaper_wtype()?;
            }
            WallpaperMethod::Ydotool => {
                change_wallpaper_ydotool()?;
            }
        }

        Ok(())
    }
}
