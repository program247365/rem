use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

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

        match eventkit.get_reminder_lists() {
            Ok(lists) => {
                self.lists = lists;
                self.loading = false;
                if !self.lists.is_empty() {
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => {
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
        let paragraph = Paragraph::new("Loading reminder lists...")
            .block(Block::default().borders(Borders::ALL).title("Rem"))
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
                Span::styled("r", Style::default().fg(Color::Green).bold()),
                Span::styled(" to retry", Style::default()),
            ]),
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

    fn render_lists(&mut self, f: &mut Frame, area: Rect) {
        if self.lists.is_empty() {
            let paragraph = Paragraph::new("No reminder lists found")
                .block(Block::default().borders(Borders::ALL).title("Rem"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, area);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        // Create list items with card-like appearance
        let items: Vec<ListItem> = self
            .lists
            .iter()
            .enumerate()
            .map(|(i, list)| {
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("●", Style::default().fg(parse_color(&list.color))),
                        Span::raw(" "),
                        Span::styled(&list.title, Style::default().bold()),
                    ]),
                    Line::from(format!("  {} reminders", list.reminder_count)),
                ];

                if i == self.selected_index {
                    lines.push(Line::from(""));
                }

                let style = if i == self.selected_index {
                    Style::default().bg(Color::DarkGray)
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
                    .title("Reminder Lists"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("❯ ");

        f.render_stateful_widget(list_widget, main_layout[0], &mut self.list_state);

        // Instructions at the bottom
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Green).bold()),
            Span::raw(" or "),
            Span::styled("↑/↓", Style::default().fg(Color::Green).bold()),
            Span::raw(" to navigate • "),
            Span::styled("Enter", Style::default().fg(Color::Green).bold()),
            Span::raw(" to select • "),
            Span::styled("q", Style::default().fg(Color::Red).bold()),
            Span::raw(" to quit"),
        ])])
        .block(Block::default().borders(Borders::ALL))
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
