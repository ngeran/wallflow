# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WallFlow is a **Rust-based** automatic wallpaper rotation system for Hyprland. It runs as a systemd user service, rotates wallpapers at configurable intervals, and features a built-in terminal UI (TUI) for management. The project migrated from shell scripts to Rust for better performance, reliability, and type safety.

## Technology Stack

- **Rust** (edition 2021) with Tokio async runtime
- **ratatui** - Terminal UI framework for the management interface
- **clap** - CLI argument parsing with derive macros
- **serde/serde_json** - JSON configuration serialization
- **tracing** - Structured logging and instrumentation
- **Hyprland/hyprpaper** - Target wallpaper system (external dependency)
- **systemd** - User service management

## Development Commands

### Development Setup

First-time setup for development:

```bash
# Ensure Rust toolchain is installed
rustc --version  # Should be >= 1.70.0
cargo --version

# Clone repository
git clone https://github.com/ngeran/wallflow.git
cd wallflow

# Build development version
cargo build

# Add to PATH for development (optional)
export PATH="$PWD/target/debug:$PATH"
# Or add to ~/.bashrc permanently
echo "export PATH=\"$PWD/target/debug:\$PATH\"" >> ~/.bashrc
```

**Note:** For active development, use `cargo run` instead of installing to avoid PATH issues. The installed binary (`~/.cargo/bin/wallflow`) is for production use, while `cargo run` uses your local changes.

### Build and Run
```bash
# Development build with debugging
cargo build

# Optimized release build (stripped binary, LTO enabled)
cargo build --release

# Run tests
cargo test

# Run directly using cargo (recommended for development)
cargo run -- run                    # Start rotation service
cargo run -- tui                    # Launch terminal UI
cargo run -- status                 # Check service status
cargo run -- install --method omarchy --interval 180  # Install service

# Use development binary directly
./target/debug/wallflow run
./target/debug/wallflow tui

# Install locally for development/testing
cargo install --path .
# Note: This installs to ~/.cargo/bin/wallflow which may not be in PATH
```

**Important:** Use `cargo run` during development to test your changes. The `cargo install` command creates a release binary in `~/.cargo/bin/` which may not be in your PATH and doesn't reflect your latest changes.

### Service Management
```bash
# After installing via cargo, manage the service
wallflow install --method omarchy --interval 180
wallflow uninstall
wallflow status
wallflow status --journal           # Include recent logs

# Systemctl commands (service must be installed first)
systemctl --user start wallflow.service
systemctl --user stop wallflow.service
systemctl --user restart wallflow.service
journalctl --user -u wallflow.service -f   # Live logs
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy -- -D warnings    # Treat warnings as errors

# Check without building
cargo check
```

### Development Troubleshooting

**Build warnings:**
- The project currently has 6 unused function warnings
- These don't affect functionality but should be cleaned up
- Run `cargo clippy` to see all warnings
- Warnings are planned to be fixed in upcoming cleanup

**PATH issues during development:**
- Use `cargo run -- run` instead of installed `wallflow run`
- Or build with `cargo build` and use `./target/debug/wallflow`
- For release testing: `./target/release/wallflow`
- If `cargo install` binary not found, see README PATH Configuration section

**Service management during development:**
```bash
# Stop the running service to test manually
systemctl --user stop wallflow.service

# Test manually without service
cargo run -- run

# Reinstall after code changes
cargo run -- install --method omarchy --interval 180

# Check service logs
journalctl --user -u wallflow.service -n 50
```

**Common development issues:**
- **Port already in use**: Service may already be running, stop it first
- **Config not found**: Service creates default config on first run
- **Permission denied on service**: Check systemd user service permissions
- **Changes not appearing**: You're testing installed binary instead of using `cargo run`

## Architecture

### Module Structure

The codebase follows a modular design with clear separation of concerns:

**Entry Point** (`src/main.rs`)
- Parses CLI commands using clap
- Delegates to appropriate subsystems (service run, TUI, install/uninstall, status)
- Handles terminal setup/teardown for TUI mode

**Configuration** (`src/config.rs`)
- JSON-based configuration stored at `~/.config/wallflow/config`
- Enums for `WallpaperMethod` (Omarchy, Hyprctl, Socket, Wtype, Ydotool) and `WallpaperDir` (Path or Omarchy)
- Auto-creates default config on first load
- Implements `Default` trait for all config values

**Service Core** (`src/service.rs`)
- `WallFlowService` manages the rotation loop
- Uses `AtomicBool` for graceful shutdown signaling
- Runs countdown timer if `log_countdown` enabled
- Calls appropriate rotation method based on configuration
- Validates wallpaper directory on startup

**Rotation Methods** (`src/methods.rs`)
- `get_random_wallpaper()` - Recursively finds images with extensions: jpg, jpeg, png, webp
- `get_monitor()` - Detects focused monitor via `hyprctl -j monitors` with JSON parsing, falls back to text parsing
- `change_wallpaper_*()` functions - Separate implementations for each method (hyprctl, socket, wtype, ydotool)
- All methods return `Result<()>` for consistent error handling

**Omarchy Integration** (`src/omarchy.rs`)
- Handles `omarchy-theme-bg-next` command execution
- Omarchy method bypasses random selection entirely

**Systemd Management** (`src/systemd.rs`)
- `install_service()` - Generates service file template, enables and starts service
- `uninstall_service()` - Stops, disables, and removes service file
- Service template uses `Type=simple` with `Restart=always` and `RestartSec=10`
- Uses `graphical-session.target` for dependency ordering

**Terminal UI** (`src/tui/`)
- `mod.rs` - Event types and EventHandler with 250ms tick rate
- `app.rs` - Application state machine with three tabs: Status, Actions, Info
- `ui.rs` - Drawing logic using ratatui widgets
- `events.rs` - Crossterm-based event handling for keyboard input

### Configuration Flow

1. Config is loaded from `~/.config/wallflow/config` or created with defaults
2. CLI install flags can override config values during installation
3. Service runs with `wallflow run` (typically started by systemd)
4. TUI reads current config but doesn't modify it directly (actions trigger external edits)

### Error Handling Pattern

- Uses `anyhow::Result<T>` for application errors
- Uses `Context` trait for error chain context: `.with_context(|| format!("..."))`
- Errors in service loop are logged but don't terminate the service
- All external commands use `std::process::Command` with output status checking

### Key Data Structures

```rust
// WallpaperMethod - determines which rotation method to use
pub enum WallpaperMethod { Omarchy, Hyprctl, Socket, Wtype, Ydotool }

// WallpaperDir - either static path or dynamic Omarchy detection
pub enum WallpaperDir { Path(PathBuf), Omarchy }

// Config - main configuration struct with serde derives
pub struct Config {
    pub interval: u64,
    pub method: WallpaperMethod,
    pub wallpaper_dir: WallpaperDir,
    pub unload_unused: bool,
    pub log_countdown: bool,
}
```

## Testing Wallpaper Changes Manually

After building, test rotation methods directly:

```bash
# Omarchy method (if installed)
omarchy-theme-bg-next

# hyprctl method
hyprctl hyprpaper wallpaper "all,/path/to/wallpaper.jpg"

# socket method (requires socat)
echo "wallpaper all,/path/to/wallpaper.jpg" | socat - "$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.hyprpaper.sock"

# wtype method (requires keybind in hyprland.conf)
wtype -M ctrl -M super P + space

# ydotool method (requires ydotool service)
ydotool key -d 100 29:1 125:1 57:1 57:0 125:0 29:0  # Ctrl+Super+Space
```

## File Locations

- **Binary**: `~/.cargo/bin/wallflow` (cargo install) or `~/.local/bin/wallflow` (manual)
- **Config**: `~/.config/wallflow/config`
- **Service**: `~/.config/systemd/user/wallflow.service`
- **Logs**: `journalctl --user -u wallflow.service`

## Common Modifications

### Adding a New Rotation Method

1. Add variant to `WallpaperMethod` enum in `config.rs`
2. Implement `change_wallpaper_*()` function in `methods.rs`
3. Add match arm in `service.rs` `change_wallpaper()` method
4. Update clap CLI in `cli.rs` if needed for install command
5. Document method-specific dependencies in README.md

### Modifying TUI

The TUI uses a 250ms tick rate. To change update frequency:
```rust
// In main.rs, Command::Tui handler
let events = EventHandler::new(250); // milliseconds
```

TUI state is updated in `App::update()` called on each tick. Service status is checked via `systemd::is_service_active()`.

### Adding Configuration Options

1. Add field to `Config` struct in `config.rs` with `serde` attributes
2. Add `default_*()` function if not Default trait
3. Update `Config::default()` implementation
4. Access via `self.config.field_name` in service methods
5. Service restart required for config changes to take effect

## Release Build Optimizations

The `Cargo.toml` release profile enables:
- `opt-level = 3` - Maximum optimization
- `lto = true` - Link-time optimization
- `codegen-units = 1` - Better optimization at cost of slower compile
- `strip = true` - Remove debug symbols from binary

This results in a small, fast binary suitable for distribution.
