use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};

use crate::config::{Config, WallpaperDir};
use crate::systemd;

/// Application tabs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Status,
    Actions,
    Info,
}

/// TUI Application state
pub struct App {
    pub should_quit: bool,
    pub current_tab: Tab,
    pub config: Config,
    pub next_change: Option<DateTime<Local>>,
    pub selected_action: usize,
    pub show_popup: bool,
    pub popup_message: String,
    pub last_update: Instant,
    pub service_start_time: Option<DateTime<Local>>,
    pub changes_today: usize,
    pub last_change_time: Option<DateTime<Local>>,
}

impl App {
    /// Create a new TUI app
    pub fn new(config: Config) -> Self {
        let is_active = systemd::is_service_active();

        Self {
            should_quit: false,
            current_tab: Tab::Status,
            next_change: None,
            selected_action: 0,
            show_popup: false,
            popup_message: String::new(),
            last_update: Instant::now(),
            service_start_time: if is_active { Some(Local::now()) } else { None },
            changes_today: 0,
            last_change_time: None,
            config,
        }
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.show_popup {
            // Dismiss popup on any key
            self.show_popup = false;
            self.popup_message.clear();
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('1') => {
                self.current_tab = Tab::Status;
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Actions;
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Info;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_tab == Tab::Actions {
                    let actions = Self::get_actions();
                    if self.selected_action > 0 {
                        self.selected_action -= 1;
                    } else {
                        self.selected_action = actions.len() - 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_tab == Tab::Actions {
                    let actions = Self::get_actions();
                    if self.selected_action < actions.len() - 1 {
                        self.selected_action += 1;
                    } else {
                        self.selected_action = 0;
                    }
                }
            }
            KeyCode::Enter => {
                if self.current_tab == Tab::Actions {
                    self.execute_action()?;
                }
            }
            KeyCode::Char('r') => {
                // Refresh
            }
            _ => {}
        }

        Ok(())
    }

    /// Update application state
    pub fn update(&mut self) {
        // Update every second
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.last_update = Instant::now();

            // Update next change time based on interval
            if systemd::is_service_active() {
                if self.service_start_time.is_none() {
                    self.service_start_time = Some(Local::now());
                }

                if self.next_change.is_none() {
                    self.next_change = Some(Local::now() + chrono::Duration::seconds(self.config.interval as i64));
                } else if let Some(next) = self.next_change {
                    // If we've passed the next change time, update it
                    if Local::now() > next {
                        self.last_change_time = Some(next);
                        self.changes_today += 1;
                        self.next_change = Some(Local::now() + chrono::Duration::seconds(self.config.interval as i64));
                    }
                }
            } else {
                // Service stopped
                self.next_change = None;
            }
        }
    }

    /// Calculate time until next change
    fn calculate_time_remaining(&self) -> Option<i64> {
        self.next_change.map(|next| {
            let now = Local::now();
            let duration = next.signed_duration_since(now);
            duration.num_seconds().max(0)
        })
    }

    /// Format seconds as MM:SS
    fn format_duration(seconds: i64) -> String {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }

    /// Format seconds as human-readable time
    fn format_human_duration(seconds: i64) -> String {
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            let minutes = seconds / 60;
            let secs = seconds % 60;
            if secs == 0 {
                format!("{}m", minutes)
            } else {
                format!("{}m {}s", minutes, secs)
            }
        } else {
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            if minutes == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}m", hours, minutes)
            }
        }
    }

    /// Get available actions with descriptions
    fn get_actions() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("▶", "Start Service", "Enable and start the wallpaper rotation service"),
            ("⏸", "Stop Service", "Stop the wallpaper rotation service"),
            ("↻", "Restart Service", "Restart the service to apply changes"),
            ("⏱", "Change Interval", "Adjust rotation frequency (edit config)"),
            ("📦", "Edit Config", "Open configuration file in editor"),
            ("📊", "View Journal", "Show service logs via journalctl"),
        ]
    }

    /// Execute selected action
    fn execute_action(&mut self) -> Result<()> {
        let actions = Self::get_actions();

        if self.selected_action >= actions.len() {
            return Ok(());
        }

        match actions[self.selected_action].1 {
            "Start Service" => {
                if let Err(e) = systemd::start_service() {
                    self.show_popup_error(&format!("Failed to start: {}", e));
                } else {
                    self.service_start_time = Some(Local::now());
                    self.next_change = Some(Local::now() + chrono::Duration::seconds(self.config.interval as i64));
                    self.show_popup_success("Service started!");
                }
            }
            "Stop Service" => {
                if let Err(e) = systemd::stop_service() {
                    self.show_popup_error(&format!("Failed to stop: {}", e));
                } else {
                    self.next_change = None;
                    self.show_popup_success("Service stopped!");
                }
            }
            "Restart Service" => {
                if let Err(e) = systemd::restart_service() {
                    self.show_popup_error(&format!("Failed to restart: {}", e));
                } else {
                    self.service_start_time = Some(Local::now());
                    self.changes_today = 0;
                    self.next_change = Some(Local::now() + chrono::Duration::seconds(self.config.interval as i64));
                    self.show_popup_success("Service restarted!");
                }
            }
            "Change Interval" => {
                // For now, show a message about editing config
                self.show_popup_info("Edit interval in config file");
            }
            "Edit Config" => {
                self.show_popup_info(&format!("Config: {}", crate::config::config_path().display()));
            }
            "View Journal" => {
                self.show_popup_info("Use: journalctl --user -u wallflow.service -f");
            }
            _ => {}
        }

        Ok(())
    }

    fn show_popup_success(&mut self, msg: &str) {
        self.show_popup = true;
        self.popup_message = format!("✓ {}", msg);
    }

    fn show_popup_error(&mut self, msg: &str) {
        self.show_popup = true;
        self.popup_message = format!("✗ {}", msg);
    }

    fn show_popup_info(&mut self, msg: &str) {
        self.show_popup = true;
        self.popup_message = format!("ℹ {}", msg);
    }

    /// Draw the UI
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Footer
            ])
            .split(size);

        self.draw_header(frame, chunks[0]);
        self.draw_main_content(frame, chunks[1]);
        self.draw_footer(frame, chunks[2]);

        // Draw popup if needed
        if self.show_popup {
            self.draw_popup(frame, size);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled("🌊 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "WallFlow TUI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let header = Paragraph::new(title)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }

    fn draw_main_content(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tab selector
                Constraint::Min(0),     // Tab content
            ])
            .split(area);

        self.draw_tab_selector(frame, chunks[0]);

        match self.current_tab {
            Tab::Status => self.draw_status_tab(frame, chunks[1]),
            Tab::Actions => self.draw_actions_tab(frame, chunks[1]),
            Tab::Info => self.draw_info_tab(frame, chunks[1]),
        }
    }

    fn draw_tab_selector(&self, frame: &mut Frame, area: Rect) {
        // Define tabs with their corresponding enum values
        let tabs = vec![
            ("1. Status", Tab::Status),
            ("2. Actions", Tab::Actions),
            ("3. Info", Tab::Info),
        ];

        let titles: Vec<Line> = tabs
            .iter()
            .map(|(text, tab)| {
                let is_selected = self.current_tab == *tab;

                // Build the styled span for each tab
                let mut spans = vec![];

                // Add the number with different color if selected
                if is_selected {
                    spans.push(Span::styled(
                        &text[0..2], // "1." or "2." or "3."
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        &text[2..], // " Status", " Actions", " Info"
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        *text,
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ));
                }

                Line::from(spans)
            })
            .collect();

        // Create the paragraph with spacing between tabs
        let tab_text = Text::from(titles);

        let tab_selector = Paragraph::new(tab_text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(tab_selector, area);
    }

    fn draw_status_tab(&self, frame: &mut Frame, area: Rect) {
        let is_active = systemd::is_service_active();
        let is_enabled = systemd::is_service_enabled();

        // Build the dashboard content
        let mut lines = vec![];

        // Title
        lines.push(Line::from(vec![
            Span::styled("                    ", Style::default()),
            Span::styled("🌊 WallFlow Dashboard", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        // Service Status section
        let status_color = if is_active { Color::Green } else { Color::Red };
        let status_text = if is_active { "▶ RUNNING" } else { "■ STOPPED" };
        let active_status = if is_active { "✓ Active" } else { "✗ Inactive" };

        lines.push(Line::from(vec![
            Span::styled("  Service Status    ", Style::default().fg(Color::White)),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(active_status, Style::default().fg(status_color)),
        ]));
        lines.push(Line::from(""));

        // Next Countdown section
        if is_active {
            if let Some(time_remaining) = self.calculate_time_remaining() {
                let formatted_time = Self::format_duration(time_remaining);
                let elapsed = (self.config.interval as i64 - time_remaining) as f64;
                let progress = if self.config.interval > 0 {
                    (elapsed / self.config.interval as f64) * 100.0
                } else {
                    0.0
                };

                lines.push(Line::from(vec![
                    Span::styled("  Next Countdown   ", Style::default().fg(Color::White)),
                ]));

                // Large countdown timer display
                lines.push(Line::from(vec![
                    Span::styled("  ┌─────────────┐  ", Style::default().fg(Color::Cyan)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  │", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("  {}  ", formatted_time),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("│  ", Style::default().fg(Color::Cyan)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  └─────────────┘  ", Style::default().fg(Color::Cyan)),
                ]));

                lines.push(Line::from(""));

                // Progress bar
                let filled = (progress / 100.0 * 30.0) as usize;
                let bar_filled = "█".repeat(filled);
                let bar_empty = "░".repeat(30 - filled);
                lines.push(Line::from(vec![
                    Span::styled("  Progress: ", Style::default().fg(Color::White)),
                    Span::styled(bar_filled, Style::default().fg(Color::Green)),
                    Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
                    Span::styled(format!(" {:.0}%", progress), Style::default().fg(Color::Green)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  Next Countdown   ", Style::default().fg(Color::White)),
                    Span::styled("Initializing...", Style::default().fg(Color::Yellow)),
                ]));
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("  Next Countdown   ", Style::default().fg(Color::DarkGray)),
                Span::styled("Service stopped", Style::default().fg(Color::Red)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ──────────────────────────────────────────────────────  ",
                Style::default().fg(Color::DarkGray)
            ),
        ]));
        lines.push(Line::from(""));

        // Configuration section
        lines.push(Line::from(vec![
            Span::styled("  Configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("  Method:        ", Style::default().fg(Color::White)),
            Span::styled(self.config.method.as_str(), Style::default().fg(Color::Cyan)),
        ]));

        let interval_str = format!(
            "{} seconds ({})",
            self.config.interval,
            Self::format_human_duration(self.config.interval as i64)
        );
        lines.push(Line::from(vec![
            Span::styled("  Interval:      ", Style::default().fg(Color::White)),
            Span::styled(interval_str, Style::default().fg(Color::Cyan)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Autostart:     ", Style::default().fg(Color::White)),
            Span::styled(
                if is_enabled { "✓ Enabled" } else { "✗ Disabled" },
                Style::default().fg(if is_enabled { Color::Green } else { Color::Red }),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Unload Unused: ", Style::default().fg(Color::White)),
            Span::styled(
                if self.config.unload_unused { "✓ Yes" } else { "✗ No" },
                Style::default().fg(if self.config.unload_unused { Color::Green } else { Color::Red }),
            ),
        ]));

        lines.push(Line::from(""));

        // Statistics section
        lines.push(Line::from(vec![
            Span::styled("  Statistics", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("  Changes Today:  ", Style::default().fg(Color::White)),
            Span::styled(format!("{}", self.changes_today), Style::default().fg(Color::Yellow)),
        ]));

        if let Some(start_time) = self.service_start_time {
            let uptime_seconds = Local::now().signed_duration_since(start_time).num_seconds().max(0);
            let uptime_str = Self::format_human_duration(uptime_seconds);
            lines.push(Line::from(vec![
                Span::styled("  Uptime:        ", Style::default().fg(Color::White)),
                Span::styled(uptime_str, Style::default().fg(Color::Yellow)),
            ]));
        }

        if let Some(last_change) = self.last_change_time {
            let last_change_str = last_change.format("%H:%M:%S").to_string();
            lines.push(Line::from(vec![
                Span::styled("  Last Change:   ", Style::default().fg(Color::White)),
                Span::styled(last_change_str, Style::default().fg(Color::Yellow)),
            ]));
        }

        // Create bordered block
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn draw_actions_tab(&self, frame: &mut Frame, area: Rect) {
        let actions = Self::get_actions();

        let mut lines = vec![];

        // Add header with keyboard hint
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "↑↓: Navigate  Enter: Execute",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(""));

        // Add each action with description
        for (i, (icon, text, description)) in actions.iter().enumerate() {
            let is_selected = i == self.selected_action;

            // Create the action line
            lines.push(Line::from(vec![
                Span::styled(
                    if is_selected { "▶ " } else { "  " },
                    Style::default().fg(if is_selected { Color::Cyan } else { Color::DarkGray }),
                ),
                Span::styled(
                    *icon,
                    Style::default()
                        .fg(if is_selected { Color::Cyan } else { Color::DarkGray })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    *text,
                    Style::default()
                        .fg(if is_selected {
                            Color::Cyan
                        } else {
                            Color::White
                        })
                        .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                ),
            ]));

            // Add description on next line with indentation
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    *description,
                    Style::default()
                        .fg(if is_selected {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        })
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));

            // Add spacing between actions
            if i < actions.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Actions ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn draw_info_tab(&self, frame: &mut Frame, area: Rect) {
        let config_path = crate::config::config_path();
        let service_path = crate::systemd::service_path();

        // Get binary path
        let binary_path = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let mut lines = vec![];

        // Version section
        lines.push(Line::from(vec![
            Span::styled("  Version: ", Style::default().fg(Color::White)),
            Span::styled("5.0.0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        // Paths section
        lines.push(Line::from(vec![
            Span::styled("  Paths", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("  Binary:    ", Style::default().fg(Color::White)),
            Span::styled(binary_path, Style::default().fg(Color::Yellow)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Config:    ", Style::default().fg(Color::White)),
            Span::styled(config_path.display().to_string(), Style::default().fg(Color::Yellow)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Service:   ", Style::default().fg(Color::White)),
            Span::styled(service_path.display().to_string(), Style::default().fg(Color::Yellow)),
        ]));

        // System section
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  System", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        // Get current wallpaper directory
        let wallpaper_dir = match &self.config.wallpaper_dir {
            WallpaperDir::Omarchy => "Omarchy (dynamic theme detection)".to_string(),
            WallpaperDir::Path(p) => p.display().to_string(),
        };

        lines.push(Line::from(vec![
            Span::styled("  Method:    ", Style::default().fg(Color::White)),
            Span::styled(self.config.method.as_str(), Style::default().fg(Color::Green)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Directory: ", Style::default().fg(Color::White)),
            Span::styled(wallpaper_dir, Style::default().fg(Color::Green)),
        ]));

        // Keybindings section
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Keybindings", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  1-3       ", Style::default().fg(Color::Yellow)),
            Span::styled("Switch between tabs", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ↑/↓ or j/k", Style::default().fg(Color::Yellow)),
            Span::styled("Navigate actions", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Enter     ", Style::default().fg(Color::Yellow)),
            Span::styled("Execute selected action", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  q/Esc     ", Style::default().fg(Color::Yellow)),
            Span::styled("Quit TUI", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  r         ", Style::default().fg(Color::Yellow)),
            Span::styled("Refresh status", Style::default().fg(Color::White)),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Information ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let time = Local::now().format("%H:%M:%S").to_string();

        // Build footer with keybinding hints and info
        let spans = vec![
            // Keybindings section
            Span::styled("1-3:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Tabs ", Style::default().fg(Color::White)),
            Span::styled("↑↓:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Nav ", Style::default().fg(Color::White)),
            Span::styled("Enter:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Exec ", Style::default().fg(Color::White)),
            Span::styled("q:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Quit ", Style::default().fg(Color::White)),
            Span::raw("│ "),
            // Version section
            Span::styled("v5.0.0", Style::default().fg(Color::Yellow)),
            Span::raw(" │ "),
            // Time section
            Span::styled(time, Style::default().fg(Color::Green)),
        ];

        let footer = Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(footer, area);
    }

    fn draw_popup(&self, frame: &mut Frame, size: Rect) {
        let popup_width = 40.min(size.width - 4);
        let popup_height = 3;

        let popup_area = Rect {
            x: (size.width - popup_width) / 2,
            y: (size.height - popup_height) / 2,
            width: popup_width,
            height: popup_height,
        };

        let popup = Paragraph::new(self.popup_message.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Center);

        frame.render_widget(popup, popup_area);
    }
}
