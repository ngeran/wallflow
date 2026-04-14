use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wallflow")]
#[command(about = "Automatic wallpaper rotation for Hyprland", long_about = None)]
#[command(version = "5.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run as background service
    Run {
        /// Path to config file (optional)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Launch TUI interface
    Tui,

    /// Install systemd service
    Install {
        /// Wallpaper rotation method
        #[arg(long, value_enum)]
        method: Option<WallpaperMethodArg>,

        /// Rotation interval in seconds
        #[arg(long)]
        interval: Option<u64>,

        /// Wallpaper directory path
        #[arg(long)]
        wallpaper_dir: Option<String>,
    },

    /// Uninstall systemd service
    Uninstall,

    /// Show service status
    Status {
        /// Show journal logs
        #[arg(long)]
        journal: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum WallpaperMethodArg {
    Omarchy,
    Hyprctl,
    Socket,
    Wtype,
    Ydotool,
}

impl From<WallpaperMethodArg> for crate::config::WallpaperMethod {
    fn from(value: WallpaperMethodArg) -> Self {
        match value {
            WallpaperMethodArg::Omarchy => crate::config::WallpaperMethod::Omarchy,
            WallpaperMethodArg::Hyprctl => crate::config::WallpaperMethod::Hyprctl,
            WallpaperMethodArg::Socket => crate::config::WallpaperMethod::Socket,
            WallpaperMethodArg::Wtype => crate::config::WallpaperMethod::Wtype,
            WallpaperMethodArg::Ydotool => crate::config::WallpaperMethod::Ydotool,
        }
    }
}

