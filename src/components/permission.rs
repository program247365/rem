use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap, BorderType},
};
use tokio::sync::mpsc::UnboundedSender;

// Macro for conditional debug logging based on DEBUG environment variable
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("DEBUG").unwrap_or_default() == "true" {
            eprintln!($($arg)*);
        }
    };
}

use crate::{
    action::Action,
    components::Component,
    config::Config,
    eventkit::{EventKitManager, PermissionStatus},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionState {
    Checking,
    NeedsPermission,
    Requesting,
    Granted,
    Denied,
    Error(String),
}

pub struct PermissionComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    state: PermissionState,
    eventkit: Option<EventKitManager>,
}

impl PermissionComponent {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            state: PermissionState::Checking,
            eventkit: None,
        }
    }

    pub fn get_state(&self) -> &PermissionState {
        &self.state
    }

    fn check_permissions(&mut self) -> Result<()> {
        // Try to initialize EventKit with better error handling
        match EventKitManager::new() {
            Ok(manager) => {
                let status = manager.check_permission_status();
                debug_log!("Debug: Permission status: {:?}", status);

                match status {
                    PermissionStatus::Authorized => {
                        debug_log!("Debug: Permission already authorized, loading lists");
                        self.state = PermissionState::Granted;
                        self.eventkit = Some(manager);
                        if let Some(tx) = &self.command_tx {
                            let _ = tx.send(Action::LoadLists);
                        }
                    }
                    PermissionStatus::NotDetermined => {
                        debug_log!("Debug: Permission not determined, needs permission");
                        self.state = PermissionState::NeedsPermission;
                        self.eventkit = Some(manager);
                    }
                    PermissionStatus::Denied => {
                        debug_log!("Debug: Permission denied");
                        self.state = PermissionState::Denied;
                    }
                    PermissionStatus::Restricted => {
                        debug_log!("Debug: Permission restricted");
                        self.state = PermissionState::Error("Reminders access is restricted".to_string());
                    }
                }
            }
            Err(e) => {
                // More specific error handling
                let error_msg = format!("EventKit initialization failed: {}", e);
                debug_log!("Debug: {}", error_msg);
                self.state = PermissionState::Error(error_msg);
            }
        }

        Ok(())
    }

    async fn request_permissions(&mut self) -> Result<()> {
        if let Some(manager) = &self.eventkit {
            self.state = PermissionState::Requesting;

            match manager.request_permission().await {
                Ok(granted) => {
                    if granted {
                        self.state = PermissionState::Granted;
                        if let Some(tx) = &self.command_tx {
                            let _ = tx.send(Action::LoadLists);
                        }
                    } else {
                        self.state = PermissionState::Denied;
                    }
                }
                Err(e) => {
                    self.state =
                        PermissionState::Error(format!("Failed to request permissions: {e}"));
                }
            }
        }

        Ok(())
    }

    pub fn get_eventkit(&self) -> Option<&EventKitManager> {
        self.eventkit.as_ref()
    }

    fn render_checking(&self, f: &mut Frame, area: Rect) {
        let checking_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "🔍 Checking permissions...",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please wait",
                Style::default().fg(Color::Gray)
            )),
        ];

        let paragraph = Paragraph::new(checking_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " 📝 Rem - Apple Reminders ",
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Blue))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_needs_permission(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "🔐 Permission Required",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Rem needs access to your Apple Reminders",
                Style::default().fg(Color::White)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "This allows the app to:",
                Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)
            )),
            Line::from(vec![
                Span::styled("  📋 ", Style::default().fg(Color::Cyan)),
                Span::styled("View your reminder lists", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("  👀 ", Style::default().fg(Color::Cyan)),
                Span::styled("Read and display your reminders", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("  ✅ ", Style::default().fg(Color::Cyan)),
                Span::styled("Mark reminders as complete", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                ),
                Span::styled(" to grant access", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "q",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                ),
                Span::styled(" to quit", Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " 🔐 Permissions Required ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_requesting(&self, f: &mut Frame, area: Rect) {
        let requesting_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Requesting permissions...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Please check the system dialog",
                Style::default().fg(Color::White)
            )),
            Line::from(Span::styled(
                "and grant access to continue",
                Style::default().fg(Color::White)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "💡 Look for the macOS permission dialog",
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
            )),
        ];

        let paragraph = Paragraph::new(requesting_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " 📝 Rem - Requesting Access ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_denied(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Access Denied",
                Style::default().fg(Color::Red).bold(),
            )),
            Line::from(""),
            Line::from("Rem cannot access your Reminders."),
            Line::from(""),
            Line::from("To grant access:"),
            Line::from("1. Open System Preferences"),
            Line::from("2. Go to Security & Privacy"),
            Line::from("3. Click on Privacy tab"),
            Line::from("4. Select Reminders from the list"),
            Line::from("5. Check the box next to Rem"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("q", Style::default().fg(Color::Red).bold()),
                Span::styled(" to quit", Style::default()),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Access Denied"),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_error(&self, f: &mut Frame, area: Rect, error: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Error",
                Style::default().fg(Color::Red).bold(),
            )),
            Line::from(""),
            Line::from(error),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("q", Style::default().fg(Color::Red).bold()),
                Span::styled(" to quit", Style::default()),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

impl Component for PermissionComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::CheckPermissions => {
                // Check permissions synchronously in the main thread
                if let Err(e) = self.check_permissions() {
                    self.state = PermissionState::Error(format!("Permission check failed: {}", e));
                }
                Ok(None)
            }
            Action::RequestPermissions => {
                let tx = self.command_tx.clone();
                let mut component = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = component.request_permissions().await {
                        if let Some(tx) = tx {
                            let _ =
                                tx.send(Action::Error(format!("Permission request failed: {e}")));
                        }
                    }
                });
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match self.state {
            PermissionState::NeedsPermission => match key.code {
                KeyCode::Enter => {
                    return Ok(Some(Action::RequestPermissions));
                }
                KeyCode::Char('q') => {
                    return Ok(Some(Action::Quit));
                }
                _ => {}
            },
            PermissionState::Denied | PermissionState::Error(_) => match key.code {
                KeyCode::Char('q') => {
                    return Ok(Some(Action::Quit));
                }
                _ => {}
            },
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Center the dialog
        let popup_area = centered_rect(60, 50, area);

        // Clear the area
        f.render_widget(Clear, popup_area);

        match &self.state {
            PermissionState::Checking => self.render_checking(f, popup_area),
            PermissionState::NeedsPermission => self.render_needs_permission(f, popup_area),
            PermissionState::Requesting => self.render_requesting(f, popup_area),
            PermissionState::Granted => {
                // Don't render anything when granted - let other components take over
            }
            PermissionState::Denied => self.render_denied(f, popup_area),
            PermissionState::Error(error) => self.render_error(f, popup_area, error),
        }

        Ok(())
    }
}

impl Clone for PermissionComponent {
    fn clone(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            config: self.config.clone(),
            state: self.state.clone(),
            eventkit: None, // Don't clone EventKit manager
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
