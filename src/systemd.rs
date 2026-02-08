use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get systemd user service directory
pub fn service_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("systemd")
        .join("user")
}

/// Get the wallflow service file path
pub fn service_path() -> PathBuf {
    service_dir().join("wallflow.service")
}

/// Service file template
const SERVICE_TEMPLATE: &str = r#"[Unit]
Description=WallFlow Wallpaper Rotation Service
After=graphical-session.target hyprpaper.service
Requires=graphical-session.target

[Service]
Type=simple
ExecStart={binary} run
Restart=always
RestartSec=10

[Install]
WantedBy=graphical-session.target
"#;

/// Install the systemd service
pub fn install_service(binary_path: &PathBuf) -> Result<()> {
    let service_dir = service_dir();
    let service_path = service_path();

    // Create service directory if it doesn't exist
    fs::create_dir_all(&service_dir)
        .with_context(|| format!("Failed to create service directory {}", service_dir.display()))?;

    // Generate service file
    let service_content = SERVICE_TEMPLATE.replace("{binary}", &binary_path.to_string_lossy());

    fs::write(&service_path, service_content)
        .with_context(|| format!("Failed to write service file to {}", service_path.display()))?;

    tracing::info!("✓ Created service file: {}", service_path.display());

    // Reload systemd
    let reload_output = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .context("Failed to reload systemd daemon")?;

    if !reload_output.status.success() {
        anyhow::bail!("Failed to reload systemd: {}", String::from_utf8_lossy(&reload_output.stderr));
    }

    tracing::info!("✓ Reloaded systemd daemon");

    // Enable service
    let enable_output = Command::new("systemctl")
        .args(["--user", "enable", "wallflow.service"])
        .output()
        .context("Failed to enable wallflow service")?;

    if !enable_output.status.success() {
        anyhow::bail!("Failed to enable service: {}", String::from_utf8_lossy(&enable_output.stderr));
    }

    tracing::info!("✓ Enabled wallflow.service");

    // Start service
    start_service()?;

    Ok(())
}

/// Uninstall the systemd service
pub fn uninstall_service() -> Result<()> {
    let service_path = service_path();

    // Stop and disable service
    let _ = Command::new("systemctl")
        .args(["--user", "stop", "wallflow.service"])
        .output();

    let _ = Command::new("systemctl")
        .args(["--user", "disable", "wallflow.service"])
        .output();

    // Remove service file
    if service_path.exists() {
        fs::remove_file(&service_path)
            .with_context(|| format!("Failed to remove service file {}", service_path.display()))?;
        tracing::info!("✓ Removed service file");
    }

    // Reload systemd
    let reload_output = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .context("Failed to reload systemd daemon")?;

    if !reload_output.status.success() {
        anyhow::bail!("Failed to reload systemd: {}", String::from_utf8_lossy(&reload_output.stderr));
    }

    tracing::info!("✓ Reloaded systemd daemon");

    Ok(())
}

/// Start the service
pub fn start_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "start", "wallflow.service"])
        .output()
        .context("Failed to start wallflow service")?;

    if !output.status.success() {
        anyhow::bail!("Failed to start service: {}", String::from_utf8_lossy(&output.stderr));
    }

    tracing::info!("✓ Started wallflow.service");
    Ok(())
}

/// Stop the service
pub fn stop_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "stop", "wallflow.service"])
        .output()
        .context("Failed to stop wallflow service")?;

    if !output.status.success() {
        anyhow::bail!("Failed to stop service: {}", String::from_utf8_lossy(&output.stderr));
    }

    tracing::info!("✓ Stopped wallflow.service");
    Ok(())
}

/// Restart the service
pub fn restart_service() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "restart", "wallflow.service"])
        .output()
        .context("Failed to restart wallflow service")?;

    if !output.status.success() {
        anyhow::bail!("Failed to restart service: {}", String::from_utf8_lossy(&output.stderr));
    }

    tracing::info!("✓ Restarted wallflow.service");
    Ok(())
}

/// Check if service is active
pub fn is_service_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "wallflow.service"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if service is enabled
pub fn is_service_enabled() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-enabled", "wallflow.service"])
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim() == "enabled"
        })
        .unwrap_or(false)
}

/// Get service status
pub fn get_service_status() -> Result<String> {
    let output = Command::new("systemctl")
        .args(["--user", "status", "wallflow.service", "--no-pager"])
        .output()
        .context("Failed to get service status")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
