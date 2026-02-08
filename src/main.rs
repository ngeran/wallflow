mod cli;
mod config;
mod methods;
mod omarchy;
mod service;
mod systemd;
mod tui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::env;
use std::path::PathBuf;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::Config;
use service::WallFlowService;
use tui::{App, EventHandler};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    match cli.command {
        Command::Run { config: config_path } => {
            // Load config
            let config = if let Some(path) = config_path {
                // Load from specified path
                let content = std::fs::read_to_string(&path)?;
                serde_json::from_str(&content)?
            } else {
                Config::load()?
            };

            // Run the service
            let service = WallFlowService::new(config);
            service.run().await?;
        }

        Command::Tui => {
            // Load config
            let config = Config::load()?;

            // Setup terminal
            enable_raw_mode()?;
            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Create app
            let mut app = App::new(config);
            let events = EventHandler::new(250); // 250ms tick rate

            // Run TUI
            loop {
                // Draw UI
                terminal.draw(|f| app.draw(f))?;

                // Handle events
                match events.next()? {
                    tui::Event::Tick => {
                        app.update();
                    }
                    tui::Event::Key(key) => {
                        app.handle_key_event(key)?;
                    }
                }

                if app.should_quit {
                    break;
                }
            }

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen
            )?;
            terminal.show_cursor()?;
        }

        Command::Install { method, interval, wallpaper_dir } => {
            // Get current executable path
            let exe_path = env::current_exe()?;
            let canonical_path = exe_path.canonicalize()?;

            // Load or create config
            let mut config = if let Ok(existing) = Config::load() {
                existing
            } else {
                Config::default()
            };

            // Apply command-line options
            if let Some(method_arg) = method {
                config.method = method_arg.into();
            }

            if let Some(interval_value) = interval {
                config.interval = interval_value;
            }

            if let Some(wp_dir) = wallpaper_dir {
                if wp_dir == "omarchy" {
                    config.wallpaper_dir = config::WallpaperDir::Omarchy;
                } else {
                    config.wallpaper_dir = config::WallpaperDir::Path(PathBuf::from(wp_dir));
                }
            }

            // Save config
            config.save()?;

            println!("Installing WallFlow service...");
            println!("Binary location: {}", canonical_path.display());
            println!("Config location: {}", config::config_path().display());
            println!();

            // Install service
            systemd::install_service(&canonical_path)?;

            println!();
            println!("✓ WallFlow service installed and started!");
            println!();
            println!("Useful commands:");
            println!("  Check status:  wallflow status");
            println!("  View logs:     journalctl --user -u wallflow.service -f");
            println!("  Stop service:  wallflow uninstall");
            println!("  Launch TUI:    wallflow tui");
        }

        Command::Uninstall => {
            println!("Uninstalling WallFlow service...");
            systemd::uninstall_service()?;
            println!("✓ WallFlow service uninstalled!");
        }

        Command::Status { journal } => {
            // Show service status
            let is_active = systemd::is_service_active();
            let is_enabled = systemd::is_service_enabled();

            println!("WallFlow Service Status:");
            println!();
            println!("  Active:   {}", if is_active { "✓ Running" } else { "✗ Stopped" });
            println!("  Enabled:  {}", if is_enabled { "✓ Yes" } else { "✗ No" });
            println!();

            if journal {
                println!("Recent journal logs:");
                println!();
                let output = std::process::Command::new("journalctl")
                    .args(["--user", "-u", "wallflow.service", "-n", "20", "--no-pager"])
                    .output()?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }

    Ok(())
}
