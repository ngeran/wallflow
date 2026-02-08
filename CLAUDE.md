# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WallFlow is a lightweight shell script-based system for automatic wallpaper rotation in **Hyprland** (Wayland compositor) with **hyprpaper** integration. It runs as a persistent systemd user service and rotates wallpapers at configurable intervals. Features **Omarchy integration** for automatic theme detection.

## Technology Stack

- **Shell Scripts**: Pure Bash for all functionality
- **Systemd**: User-level service management (`~/.config/systemd/user/`)
- **Hyprland**: Target Wayland compositor
- **hyprpaper**: Wallpaper utility for Hyprland
- **socat**: For direct socket communication with hyprpaper (socket method)
- **ydotool**: For simulating keypresses (keypress method alternative)
- **jq**: Optional JSON parser for better monitor detection

## Installation and Development Commands

### Installation
```bash
chmod +x install.sh
./install.sh
```
The interactive installer handles dependency checking, configuration, and service setup.

### Manual Testing
```bash
# Run the script directly to test before using service
~/.local/bin/wallflow.sh

# Test hyprpaper socket communication manually
hyprctl hyprpaper preload "/path/to/wallpaper.jpg"
hyprctl hyprpaper wallpaper "all,/path/to/wallpaper.jpg"
```

### Service Management
```bash
# Start/stop/restart service
systemctl --user start wallflow.service
systemctl --user stop wallflow.service
systemctl --user restart wallflow.service

# Enable/disable at login
systemctl --user enable wallflow.service
systemctl --user disable wallflow.service

# View status and logs
systemctl --user status wallflow.service
journalctl --user -u wallflow.service -f      # Live follow
journalctl --user -u wallflow.service -n 50   # Last 50 lines
```

### Debugging
```bash
chmod +x status.sh
./status.sh
```
The status script shows service state, configuration, dependency checks, and recent activity.

### Syntax Checking
```bash
bash -n ~/.local/bin/wallflow.sh
```

## Architecture

### Core Components

**wallflow.sh** - Main wallpaper rotation script
- Configuration variables at the top (INTERVAL, WALLPAPER_DIR, METHOD, UNLOAD_UNUSED)
- Three operational methods: `hyprctl` (recommended), `socket`, or `keypress`
- **Omarchy integration**: Set `WALLPAPER_DIR="omarchy"` for dynamic theme detection
- Modular function design: `get_random_wallpaper()`, `get_wallpaper_dir()`, `change_wallpaper_hyprctl()`, etc.
- Countdown logging with configurable verbosity
- Monitor auto-detection with jq fallback to text parsing
- Optional RAM saving via `hyprctl hyprpaper unload all`

**wallflow.service** - systemd user service file
- `Type=simple` for continuous operation
- `Restart=always` for automatic recovery
- `RestartSec=10` delay before restart
- `After=graphical-session.target` ensures Hyprland is ready

**install.sh** - Interactive installer
- Checks dependencies (socat/ydotool based on method selection)
- Configures script settings via sed substitution
- Creates directories and installs files
- Enables and starts the service
- Provides detailed installation report

### Wallpaper Change Methods

WallFlow supports three wallpaper change mechanisms:

1. **hyprctl Method (Recommended)**: Uses `hyprctl hyprpaper` commands directly. No additional dependencies. Most reliable for hyprpaper.

2. **Socket Method**: Uses `socat` to communicate directly with hyprpaper's Unix socket at `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.hyprpaper.sock`. Requires `socat`.

3. **Keypress Method**: Simulates Ctrl+Super+Space keybind using `ydotool`. Works with any wallpaper manager but requires ydotool service to be running.

### Omarchy Integration

When `WALLPAPER_DIR` is set to `"omarchy"`, WallFlow dynamically detects the current theme:
- Reads theme from `~/.config/omarchy/theme`
- Builds path: `~/.config/omarchy/backgrounds/{theme}/`
- Automatically adapts when switching themes
- Falls back to base directory if theme path doesn't exist

The `get_wallpaper_dir()` function handles this logic and is called by `get_random_wallpaper()`.

### Configuration

Key configuration variables in `wallflow.sh`:
- `INTERVAL` - Rotation interval in seconds (default: 180)
- `WALLPAPER_DIR` - Path to wallpaper directory, or `"omarchy"` for dynamic Omarchy theme detection
- `METHOD` - "hyprctl", "socket", or "keypress"
- `UNLOAD_UNUSED` - Set to `true` to unload old wallpapers and save RAM
- `LOG_COUNTDOWN` - Enable/disable countdown logging
- `OMARCHY_BASE_DIR` - Base path for Omarchy themes (default: `~/.config/omarchy/backgrounds`)
- `OMARCHY_THEME_FILE` - File containing current theme name (default: `~/.config/omarchy/theme`)

The installer uses `sed` to modify these values during installation:
```bash
sed -i "s/^METHOD=.*/METHOD=\"$METHOD\"/" "$SCRIPT_NAME"
sed -i "s|^WALLPAPER_DIR=.*|WALLPAPER_DIR=\"$WALLPAPER_DIR\"|" "$SCRIPT_NAME"
sed -i "s/^INTERVAL=.*/INTERVAL=$INTERVAL/" "$SCRIPT_NAME"
sed -i "s/^UNLOAD_UNUSED=.*/UNLOAD_UNUSED=$UNLOAD_UNUSED/" "$SCRIPT_NAME"
```

### Multi-Monitor Support

The script auto-detects monitors via:
1. `jq` parsing of `hyprctl -j monitors` (prioritizes focused monitor)
2. Fallback to text parsing with `awk`

For manual monitor specification, modify the hyprctl commands in `change_wallpaper_hyprctl()`:
```bash
# All monitors
hyprctl hyprpaper wallpaper "all,$wallpaper"

# Specific monitor
hyprctl hyprpaper wallpaper "DP-1,$wallpaper"

# Different wallpapers per monitor
hyprctl hyprpaper wallpaper "DP-1,$wallpaper1"
hyprctl hyprpaper wallpaper "HDMI-A-1,$wallpaper2"
```

## File Locations After Installation

- **Script**: `~/.local/bin/wallflow.sh`
- **Service**: `~/.config/systemd/user/wallflow.service`
- **Logs**: `journalctl --user -u wallflow.service`

## Supported Image Formats

The script searches for: `.jpg`, `.jpeg`, `.png`, `.webp` (case-insensitive)

## RAM Optimization

The `UNLOAD_UNUSED` option controls memory usage:
- When `true`: Runs `hyprctl hyprpaper unload all` before each change, freeing memory from old wallpapers
- When `false`: Keeps all loaded wallpapers in memory for faster switching

Recommended to enable (`true`) if you have limited RAM or many high-resolution wallpapers.

## Modification Patterns

When modifying the script:
1. Configuration changes require service restart: `systemctl --user restart wallflow.service`
2. The installer uses sed patterns - keep config variable formats consistent
3. All functions are modular - add new functionality as separate functions
4. Logs use timestamp format: `[$(date '+%Y-%m-%d %H:%M:%S')]`
