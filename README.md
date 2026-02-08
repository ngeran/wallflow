# 🌊 WallFlow

**Automatic wallpaper rotation for Hyprland with Omarchy integration**

WallFlow is a lightweight, high-performance wallpaper rotation system for Hyprland. Written in Rust for speed and reliability, it runs as a persistent systemd service and rotates wallpapers at configurable intervals.

---

## 🎯 Why WallFlow?

WallFlow was built to solve a simple problem: **automatically cycling through beautiful wallpapers** without manual intervention. Whether you're using:

- **Omarchy** for dynamic theming
- **hyprpaper** for wallpaper management
- **Custom keybinds** for wallpaper switching

WallFlow handles it all with a single, efficient binary.

### Why Rust?

After migrating from shell scripts, we chose Rust for:
- **Performance**: Native binary with instant startup
- **Reliability**: Compile-time error checking prevents runtime issues
- **No Dependencies**: Single binary, no Python or Bash required
- **Built-in TUI**: Beautiful terminal interface for management
- **Type Safety**: Catch errors before they happen

---

## ✨ Features

- **🚀 Native Performance**: Written in Rust for fast startup and low resource usage
- **🎨 Beautiful TUI**: Terminal UI for real-time monitoring and control
- **🔄 Multiple Rotation Methods**: Omarchy, hyprctl, socket, wtype, ydotool
- **⚙️ Simple Configuration**: JSON-based config file
- **🔔 Service Integration**: Systemd user service for background operation
- **🎯 Omarchy Support**: Dynamic theme detection and automatic wallpaper switching
- **📊 Dashboard**: Real-time countdown, progress bar, and statistics

---

## 📋 Prerequisites

### Required
- **Hyprland** (Wayland compositor)
- **hyprpaper** (Hyprland wallpaper utility)
- **Systemd** (user service management)
- **Rust toolchain** (for building from source)

### Method-Specific Dependencies

| Method | Description | Dependencies |
|--------|-------------|--------------|
| `omarchy` | Uses Omarchy command directly | Omarchy installed |
| `hyprctl` | Direct hyprpaper control | None (built into hyprpaper) |
| `socket` | Socket communication with hyprpaper | `socat` |
| `wtype` | Simulates keybind (Wayland-native) | `wtype` |
| `ydotool` | Simulates keybind (X11/Wayland) | `ydotool` + ydotool service |

Install dependencies as needed:
```bash
# For socket method
sudo pacman -S socat

# For wtype method (Wayland, recommended)
sudo pacman -S wtype

# For ydotool method
sudo pacman -S ydotool
sudo systemctl enable --now ydotool
```

---

## 🚀 Installation

### Method 1: Install from Cargo (Recommended)

```bash
cargo install wallflow
wallflow install --method omarchy --interval 180
```

### Method 2: Build from Source

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/wallflow.git
   cd wallflow
   ```

2. **Build the release binary:**
   ```bash
   cargo build --release
   ```

3. **Install the binary:**
   ```bash
   cp target/release/wallflow ~/.local/bin/
   chmod +x ~/.local/bin/wallflow
   ```

4. **Install and start the service:**
   ```bash
   wallflow install --method omarchy --interval 180
   ```

### Method 3: Using cargo install --path

If you have the repository locally:
```bash
cargo install --path .
wallflow install --method omarchy --interval 180
```

---

## ✅ Verify Installation

After installation, verify everything is working:

### 1. Check the binary
```bash
which wallflow
# Should output: /home/your-user/.cargo/bin/wallflow or /home/your-user/.local/bin/wallflow

wallflow --version
# Should output: wallflow 5.0.0
```

### 2. Check the service
```bash
systemctl --user status wallflow.service
```

You should see:
- **Active: active (running)** - Service is running
- **Loaded: loaded** - Service file is installed

### 3. Test the TUI
```bash
wallflow tui
```

You should see the TUI interface with:
- Status dashboard showing countdown and progress
- Actions tab for service management
- Info tab with configuration details

Press `q` to exit.

### 4. View live logs
```bash
journalctl --user -u wallflow.service -f
```

You should see log entries showing wallpaper rotations.

### 5. Verify configuration
```bash
cat ~/.config/wallflow/config
```

Should show your JSON configuration.

---

## 🎮 Usage

### Command-Line Interface

```bash
# Launch TUI interface
wallflow tui

# Run as background service (usually started by systemd)
wallflow run

# Install systemd service with custom settings
wallflow install --method omarchy --interval 180

# Uninstall systemd service
wallflow uninstall

# Show service status
wallflow status

# Show service status with recent logs
wallflow status --journal
```

### TUI Interface

Launch with `wallflow tui` for an interactive management interface.

**Keybindings:**
- `1`, `2`, `3` - Switch tabs (Status, Actions, Info)
- `↑`, `↓` or `j`, `k` - Navigate actions
- `Enter` - Execute selected action
- `q` or `Esc` - Quit

**Tabs:**

**1. Status Dashboard**
- Service status (Running/Stopped)
- Countdown timer to next change
- Progress bar showing elapsed time
- Configuration overview
- Statistics (changes today, uptime, last change)

**2. Actions**
- Start/Stop/Restart service
- Edit configuration
- View journal logs
- Change interval (via config edit)

**3. Info**
- Version information
- Binary, config, and service file locations
- System information
- Keybinding reference

### Service Management

```bash
# Check status
systemctl --user status wallflow.service

# Start/Stop/Restart
systemctl --user start wallflow.service
systemctl --user stop wallflow.service
systemctl --user restart wallflow.service

# Enable/disable at login
systemctl --user enable wallflow.service
systemctl --user disable wallflow.service
```

---

## ⚙️ Configuration

Configuration is stored in `~/.config/wallflow/config` as JSON:

```json
{
  "interval": 180,
  "method": "omarchy",
  "wallpaper_dir": "omarchy",
  "unload_unused": true,
  "log_countdown": true
}
```

### Configuration Options

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `interval` | number | Rotation interval in seconds | `180` (3 minutes) |
| `method` | string | Rotation method (see below) | `"omarchy"` |
| `wallpaper_dir` | string/object | `"omarchy"` or path to directory | `"omarchy"` |
| `unload_unused` | boolean | Unload old wallpapers to save RAM | `true` |
| `log_countdown` | boolean | Log countdown timer to journal | `true` |

### Rotation Methods

- **`"omarchy"`** - Uses `omarchy-theme-bg-next` command directly (recommended for Omarchy users)
- **`"hyprctl"`** - Uses `hyprctl hyprpaper` commands directly
- **`"socket"`** - Direct socket communication with hyprpaper (requires `socat`)
- **`"wtype"`** - Simulates Ctrl+Super+Space keybind with wtype (requires `wtype`)
- **`"ydotool"`** - Simulates keybind with ydotool (requires `ydotool`)

### Wallpaper Directory Options

- **`"omarchy"`** - Auto-detect theme from Omarchy (recommended)
- **`"/path/to/wallpapers"`** - Static directory path

### Edit Configuration

Edit the config file directly:
```bash
nano ~/.config/wallflow/config
```

Then restart the service:
```bash
systemctl --user restart wallflow.service
```

Or use the TUI (Actions tab → Edit Config).

---

## 🗑️ Uninstallation

### Automated Uninstallation

```bash
wallflow uninstall
```

This will:
1. Stop and disable the service
2. Remove the service file
3. Reload systemd daemon

### Manual Uninstallation

```bash
# Stop and disable service
systemctl --user stop wallflow.service
systemctl --user disable wallflow.service

# Remove service file
rm ~/.config/systemd/user/wallflow.service

# Remove configuration
rm -r ~/.config/wallflow

# Remove binary (choose based on your installation)
rm ~/.local/bin/wallflow
# or
cargo uninstall wallflow

# Reload systemd
systemctl --user daemon-reload
systemctl --user reset-failed
```

---

## ✅ Verify Removal

After uninstallation, verify complete removal:

### 1. Check binary is removed
```bash
which wallflow
# Should output: wallflow not found
```

### 2. Check service is removed
```bash
systemctl --user status wallflow.service
# Should output: Unit wallflow.service could not be found
```

### 3. Check service file is removed
```bash
ls ~/.config/systemd/user/wallflow.service
# Should output: No such file or directory
```

### 4. Check config is removed
```bash
ls ~/.config/wallflow/config
# Should output: No such file or directory
```

### 5. Reset systemd (clean up any残留 references)
```bash
systemctl --user daemon-reload
systemctl --user reset-failed
systemctl --user list-units | grep wallflow
# Should output: (nothing)
```

---

## 🔧 Troubleshooting

### Wallpapers aren't changing

**Symptoms:** Service is running but wallpapers stay the same.

**Solutions:**

1. **Check the logs:**
   ```bash
   journalctl --user -u wallflow.service -f
   ```
   Look for error messages or failed rotation attempts.

2. **Check the TUI:**
   ```bash
   wallflow tui
   ```
   Go to Status tab to verify configuration and service state.

3. **Test your rotation method manually:**

   **For Omarchy method:**
   ```bash
   omarchy-theme-bg-next
   ```
   If this doesn't work, Omarchy may not be configured correctly.

   **For hyprctl method:**
   ```bash
   hyprctl hyprpaper wallpaper "all,/path/to/wallpaper.jpg"
   ```

   **For socket method:**
   ```bash
   # Check if socat is installed
   which socat
   ```

   **For wtype/ydotool method:**
   - Verify your keybind is configured in Hyprland
   - Check if wtype or ydotool is installed
   - For ydotool, ensure the service is running:
     ```bash
     systemctl status ydotool
     ```

4. **Check wallpaper directory:**
   ```bash
   # For Omarchy mode, check if backgrounds exist
   ls ~/.config/omarchy/backgrounds/$(cat ~/.config/omarchy/theme)/

   # For static directory, check if it has images
   ls ~/.config/wallpapers/
   ```

### Service fails to start

**Symptoms:** `systemctl --user start wallflow.service` fails immediately.

**Solutions:**

1. **Check detailed error logs:**
   ```bash
   journalctl --user -u wallflow.service -n 50 --no-pager
   ```

2. **Test running directly (not as service):**
   ```bash
   wallflow run
   ```
   This will show errors directly to terminal.

3. **Verify binary is executable:**
   ```bash
   ls -l ~/.local/bin/wallflow
   # Should show: -rwxr-xr-x (executable)
   chmod +x ~/.local/bin/wallflow
   ```

4. **Check if binary exists in PATH:**
   ```bash
   which wallflow
   ```

5. **Verify configuration file syntax:**
   ```bash
   cat ~/.config/wallflow/config
   ```
   Ensure it's valid JSON. Use a JSON validator if needed.

### Build issues

**Symptoms:** `cargo build --release` fails with errors.

**Solutions:**

1. **Update Rust toolchain:**
   ```bash
   rustup update
   rustup update stable
   ```

2. **Clean and rebuild:**
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Check Rust version:**
   ```bash
   rustc --version
   # Should be at least 1.70.0
   ```

4. **Update dependencies:**
   ```bash
   cargo update
   ```

### TUI not showing updates

**Symptoms:** TUI shows old data or doesn't update.

**Solutions:**

1. **Quit and restart TUI:**
   Press `q` to quit, then run `wallflow tui` again.

2. **Check if you're running the latest binary:**
   ```bash
   which wallflow
   wallflow --version
   ```

3. **Rebuild and reinstall:**
   ```bash
   cargo build --release
   cargo install --path .
   ```

### Service restarts repeatedly

**Symptoms:** Service shows as "running" but keeps restarting.

**Solutions:**

1. **Check for crash loops:**
   ```bash
   journalctl --user -u wallflow.service -n 100
   ```
   Look for repeated start/stop patterns.

2. **Test manually to catch the error:**
   ```bash
   wallflow run
   ```

3. **Check resource usage:**
   ```bash
   # If using hyprpaper with many wallpapers, memory may be an issue
   # Enable unload_unused in config to save RAM
   ```

### Omarchy mode not working

**Symptoms:** Using Omarchy method but wallpapers don't change.

**Solutions:**

1. **Verify Omarchy is installed:**
   ```bash
   which omarchy-theme-bg-next
   ```

2. **Test Omarchy command manually:**
   ```bash
   omarchy-theme-bg-next
   ```

3. **Check Omarchy configuration:**
   ```bash
   cat ~/.config/omarchy/theme
   ls ~/.config/omarchy/backgrounds/
   ```

4. **Ensure theme has backgrounds:**
   ```bash
   ls ~/.config/omarchy/backgrounds/$(cat ~/.config/omarchy/theme)/
   ```

### Configuration changes not taking effect

**Symptoms:** You edited the config but nothing changed.

**Solutions:**

1. **Restart the service:**
   ```bash
   systemctl --user restart wallflow.service
   ```

2. **Verify config syntax:**
   ```bash
   cat ~/.config/wallflow/config
   ```
   Check for JSON syntax errors.

3. **Check if config is being read:**
   ```bash
   journalctl --user -u wallflow.service -n 20
   ```

---

## 📁 File Locations

After installation, WallFlow creates these files:

| File | Location | Purpose |
|------|----------|---------|
| Binary | `~/.local/bin/wallflow` or `~/.cargo/bin/wallflow` | Main executable |
| Config | `~/.config/wallflow/config` | JSON configuration file |
| Service | `~/.config/systemd/user/wallflow.service` | Systemd service file |
| Logs | `journalctl --user -u wallflow.service` | Service logs |

---

## 💡 Tips

- **Use the TUI** for easy management - `wallflow tui`
- **Use Omarchy mode** if you're using Omarchy - automatic theme detection
- **Use longer intervals** (5-15 minutes) to avoid distraction
- **Watch it work:** `journalctl --user -u wallflow.service -f` shows real-time changes
- **Enable `unload_unused`** if you have limited RAM or many high-res wallpapers
- **Set a Hyprland keybind** to launch TUI quickly:
  ```bash
  # Add to ~/.config/hypr/hyprland.conf
  bind = SUPER, W, exec, wallflow tui
  ```

---

## 🚀 Quick Start

```bash
# Build and install from source
git clone https://github.com/your-username/wallflow.git
cd wallflow
cargo build --release
cp target/release/wallflow ~/.local/bin/
chmod +x ~/.local/bin/wallflow

# Install service with Omarchy method
wallflow install --method omarchy --interval 180

# Launch TUI to monitor
wallflow tui

# Watch it work in real-time
journalctl --user -u wallflow.service -f
```

Or using cargo:
```bash
cargo install wallflow
wallflow install --method omarchy --interval 180
wallflow tui
```

---

## 📝 Development

### Building from source

```bash
# Clone repository
git clone https://github.com/your-username/wallflow.git
cd wallflow

# Build release version
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

### Project structure

```
wallflow/
├── src/
│   ├── main.rs      # Entry point
│   ├── cli.rs       # Command-line interface
│   ├── config.rs    # Configuration management
│   ├── service.rs   # Core service logic
│   ├── methods.rs   # Wallpaper rotation methods
│   ├── systemd.rs   # Systemd service management
│   ├── omarchy.rs   # Omarchy integration
│   └── tui/         # Terminal UI
│       ├── mod.rs
│       ├── app.rs
│       ├── events.rs
│       └── ui.rs
├── Cargo.toml       # Rust dependencies
└── README.md        # This file
```

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 🙏 Acknowledgments

- **Hyprland** - The amazing Wayland compositor
- **hyprpaper** - Wallpaper utility for Hyprland
- **Omarchy** - Dynamic theme system for Hyprland
- **Rust community** - Excellent tools and libraries

---

**Enjoy your flowing wallpapers! 🌊**
