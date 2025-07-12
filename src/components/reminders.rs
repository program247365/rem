use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, BorderType, Padding},
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
    eventkit::{EventKitManager, Reminder},
};

pub struct RemindersComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    reminders: Vec<Reminder>,
    selected_index: usize,
    list_state: ListState,
    loading: bool,
    error: Option<String>,
    list_id: String,
    list_title: String,
}

impl RemindersComponent {
    pub fn new(list_id: String, list_title: String) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            command_tx: None,
            config: Config::default(),
            reminders: Vec::new(),
            selected_index: 0,
            list_state,
            loading: false,
            error: None,
            list_id,
            list_title,
        }
    }

    pub fn load_reminders(&mut self, eventkit: &EventKitManager) -> Result<()> {
        self.loading = true;
        self.error = None;
        debug_log!("Debug: Loading reminders for list: {}", self.list_id);

        match eventkit.get_reminders_for_list(&self.list_id) {
            Ok(reminders) => {
                debug_log!("Debug: Loaded {} reminders for list '{}'", reminders.len(), self.list_title);
                if std::env::var("DEBUG").unwrap_or_default() == "true" {
                    for (i, reminder) in reminders.iter().enumerate() {
                        eprintln!("Debug: Reminder {}: {} (completed: {})", i, reminder.title, reminder.completed);
                    }
                }
                self.reminders = reminders;
                self.loading = false;
                if !self.reminders.is_empty() {
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => {
                debug_log!("Debug: Failed to load reminders: {}", e);
                self.error = Some(format!("Failed to load reminders: {e}"));
                self.loading = false;
            }
        }

        Ok(())
    }

    fn move_up(&mut self) {
        if self.reminders.is_empty() {
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.reminders.len() - 1;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn move_down(&mut self) {
        if self.reminders.is_empty() {
            return;
        }

        if self.selected_index < self.reminders.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn toggle_completed(&mut self) {
        if let Some(reminder) = self.reminders.get_mut(self.selected_index) {
            reminder.completed = !reminder.completed;
            // In a real app, you'd also update this in EventKit
        }
    }

    fn render_loading(&self, f: &mut Frame, area: Rect) {
        let loading_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Loading reminders...",
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

        let paragraph = Paragraph::new(loading_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        format!(" 📋 {} ", self.list_title),
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

    fn render_error(&self, f: &mut Frame, area: Rect, error: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️  Error Loading Reminders",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                error,
                Style::default().fg(Color::White)
            )),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                ),
                Span::styled(" to retry", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                ),
                Span::styled(" to go back", Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        format!(" ❌ {} ", self.list_title),
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Red))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_reminders(&mut self, f: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .margin(1)
            .split(area);

        if self.reminders.is_empty() {
            let empty_text = vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "✅ All done!",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "No reminders in this list",
                    Style::default().fg(Color::Gray)
                )),
            ];

            let paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(
                            format!(" 📋 {} ", self.list_title),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD)
                        ))
                        .title_alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Blue))
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, main_layout[0]);
        } else {
            // Create beautiful reminder items with enhanced styling
            let items: Vec<ListItem> = self
                .reminders
                .iter()
                .enumerate()
                .map(|(i, reminder)| {
                    let is_selected = i == self.selected_index;
                    
                    let completion_symbol = if reminder.completed { "✅" } else { "⭕" };
                    let completion_style = if reminder.completed {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    };

                    let title_style = if reminder.completed {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::CROSSED_OUT | Modifier::ITALIC)
                    } else if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD)
                    };

                    let priority_symbol = match reminder.priority {
                        1 => "🔴",
                        2 => "🟡",
                        3 => "🟢",
                        _ => "",
                    };

                    let mut lines = vec![Line::from(vec![
                        Span::styled(
                            if is_selected { "▶ " } else { "  " },
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        ),
                        Span::styled(completion_symbol, completion_style),
                        Span::raw("  "),
                        Span::styled(&reminder.title, title_style),
                        if !priority_symbol.is_empty() {
                            Span::styled(
                                format!(" {priority_symbol}"),
                                Style::default()
                            )
                        } else {
                            Span::raw("")
                        },
                    ])];

                    if let Some(notes) = &reminder.notes {
                        if !notes.is_empty() {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(
                                    format!("💭 {notes}"),
                                    Style::default().fg(if is_selected {
                                        Color::Gray
                                    } else {
                                        Color::DarkGray
                                    }).add_modifier(Modifier::ITALIC)
                                )
                            ]));
                        }
                    }

                    if let Some(due_date) = &reminder.due_date {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled("📅 Due: ", Style::default().fg(Color::Yellow)),
                            Span::styled(
                                due_date,
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            ),
                        ]));
                    }

                    // Add spacing between items
                    if i < self.reminders.len() - 1 {
                        lines.push(Line::from(""));
                    }

                    let style = if is_selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                    } else {
                        Style::default()
                    };

                    ListItem::new(lines).style(style)
                })
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(
                            format!(" 📋 {} ", self.list_title),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD)
                        ))
                        .title_alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Blue))
                        .padding(Padding::horizontal(1))
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                );

            f.render_stateful_widget(list_widget, main_layout[0], &mut self.list_state);
        }

        // Enhanced instructions at the bottom
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" toggle  ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" back  ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" quit", Style::default().fg(Color::Gray)),
        ])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " Controls ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                ))
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow))
        )
        .alignment(Alignment::Center);

        f.render_widget(instructions, main_layout[1]);
    }
}

impl Component for RemindersComponent {
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
            Action::LoadReminders(list_id) => {
                if list_id == self.list_id {
                    // This will be handled by the parent component that has access to EventKit
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.loading {
            return Ok(None);
        }

        if self.error.is_some() {
            match key.code {
                KeyCode::Char('r') => {
                    return Ok(Some(Action::LoadReminders(self.list_id.clone())));
                }
                KeyCode::Esc => {
                    return Ok(Some(Action::Back));
                }
                _ => {}
            }
            return Ok(None);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Ok(None)
            }
            KeyCode::Char(' ') => {
                self.toggle_completed();
                Ok(None)
            }
            KeyCode::Esc => Ok(Some(Action::Back)),
            KeyCode::Char('q') => Ok(Some(Action::Quit)),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        if self.loading {
            self.render_loading(f, area);
        } else if let Some(error) = &self.error {
            self.render_error(f, area, error);
        } else {
            self.render_reminders(f, area);
        }

        Ok(())
    }
}
