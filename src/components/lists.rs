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
    eventkit::{EventKitManager, ReminderList},
};

pub struct ListsComponent {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    lists: Vec<ReminderList>,
    selected_index: usize,
    list_state: ListState,
    loading: bool,
    error: Option<String>,
}

impl ListsComponent {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            command_tx: None,
            config: Config::default(),
            lists: Vec::new(),
            selected_index: 0,
            list_state,
            loading: false,
            error: None,
        }
    }

    pub fn load_lists(&mut self, eventkit: &EventKitManager) -> Result<()> {
        self.loading = true;
        self.error = None;
        debug_log!("Debug: Loading lists...");

        match eventkit.get_reminder_lists() {
            Ok(lists) => {
                debug_log!("Debug: Loaded {} lists", lists.len());
                if std::env::var("DEBUG").unwrap_or_default() == "true" {
                    for (i, list) in lists.iter().enumerate() {
                        eprintln!("Debug: List {}: {} ({} reminders)", i, list.title, list.reminder_count);
                    }
                }
                self.lists = lists;
                self.loading = false;
                if !self.lists.is_empty() {
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => {
                debug_log!("Debug: Failed to load lists: {}", e);
                self.error = Some(format!("Failed to load lists: {e}"));
                self.loading = false;
            }
        }

        Ok(())
    }

    fn move_up(&mut self) {
        if self.lists.is_empty() {
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.lists.len() - 1;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn move_down(&mut self) {
        if self.lists.is_empty() {
            return;
        }

        if self.selected_index < self.lists.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.selected_index));
    }

    fn select_current(&self) -> Option<Action> {
        if let Some(list) = self.lists.get(self.selected_index) {
            Some(Action::SelectList(list.id.clone()))
        } else {
            None
        }
    }

    fn render_loading(&self, f: &mut Frame, area: Rect) {
        let loading_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "✨ Loading your reminders...",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
            Line::from(Span::styled(
                "⏳ Please wait",
                Style::default().fg(Color::Gray)
            )),
        ];

        let paragraph = Paragraph::new(loading_text)
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

    fn render_error(&self, f: &mut Frame, area: Rect, error: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️  Error",
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
                        " ❌ Error ",
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

    fn render_lists(&mut self, f: &mut Frame, area: Rect) {
        if self.lists.is_empty() {
            let empty_text = vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "📭 No reminder lists found",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Check your Apple Reminders app",
                    Style::default().fg(Color::Gray)
                )),
            ];

            let paragraph = Paragraph::new(empty_text)
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
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .margin(1)
            .split(area);

        // Create beautiful list items with enhanced styling
        let items: Vec<ListItem> = self
            .lists
            .iter()
            .enumerate()
            .map(|(i, list)| {
                let is_selected = i == self.selected_index;
                let color = parse_color(&list.color);
                
                // Format reminder count with better styling
                let count_text = if list.reminder_count == 0 {
                    "Empty".to_string()
                } else if list.reminder_count == 1 {
                    "1 reminder".to_string()
                } else {
                    format!("{} reminders", list.reminder_count)
                };

                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            if is_selected { "▶ " } else { "  " },
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        ),
                        Span::styled(
                            "●",
                            Style::default()
                                .fg(color)
                                .add_modifier(Modifier::BOLD)
                        ),
                        Span::raw("  "),
                        Span::styled(
                            &list.title,
                            Style::default()
                                .fg(if is_selected { Color::White } else { Color::LightBlue })
                                .add_modifier(if is_selected { 
                                    Modifier::BOLD | Modifier::UNDERLINED 
                                } else { 
                                    Modifier::BOLD 
                                })
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            count_text,
                            Style::default().fg(if list.reminder_count == 0 {
                                Color::DarkGray
                            } else if is_selected {
                                Color::Gray
                            } else {
                                Color::Gray
                            })
                        ),
                    ]),
                ];

                // Add spacing between items
                if i < self.lists.len() - 1 {
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
                        " 📝 Your Reminder Lists ",
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

        // Enhanced instructions at the bottom
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("⏎", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" select  ", Style::default().fg(Color::Gray)),
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

impl Component for ListsComponent {
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
            Action::LoadLists => {
                // This will be handled by the parent component that has access to EventKit
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
                    return Ok(Some(Action::LoadLists));
                }
                KeyCode::Char('q') => {
                    return Ok(Some(Action::Quit));
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
            KeyCode::Enter => Ok(self.select_current()),
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
            self.render_lists(f, area);
        }

        Ok(())
    }
}

fn parse_color(color_str: &str) -> Color {
    // Simple color parsing - in a real app you'd want more sophisticated parsing
    match color_str {
        s if s.starts_with("#") => {
            // Try to parse hex color
            if let Ok(hex) = u32::from_str_radix(&s[1..], 16) {
                let r = ((hex >> 16) & 0xFF) as u8;
                let g = ((hex >> 8) & 0xFF) as u8;
                let b = (hex & 0xFF) as u8;
                Color::Rgb(r, g, b)
            } else {
                Color::Blue
            }
        }
        _ => Color::Blue,
    }
}
