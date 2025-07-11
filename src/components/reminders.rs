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

    pub async fn load_reminders(&mut self, eventkit: &EventKitManager) -> Result<()> {
        self.loading = true;
        self.error = None;

        match eventkit.get_reminders_for_list(&self.list_id) {
            Ok(reminders) => {
                self.reminders = reminders;
                self.loading = false;
                if !self.reminders.is_empty() {
                    self.selected_index = 0;
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => {
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
        let paragraph = Paragraph::new("Loading reminders...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.list_title.clone()),
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
                Span::styled("r", Style::default().fg(Color::Green).bold()),
                Span::styled(" to retry", Style::default()),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("Esc", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" to go back", Style::default()),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.list_title.clone()),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_reminders(&mut self, f: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        if self.reminders.is_empty() {
            let paragraph = Paragraph::new("No reminders found")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(self.list_title.clone()),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, main_layout[0]);
        } else {
            // Create list items for reminders
            let items: Vec<ListItem> = self
                .reminders
                .iter()
                .enumerate()
                .map(|(i, reminder)| {
                    let completion_symbol = if reminder.completed { "✓" } else { "○" };
                    let completion_style = if reminder.completed {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let title_style = if reminder.completed {
                        Style::default().fg(Color::Gray).crossed_out()
                    } else {
                        Style::default()
                    };

                    let priority_symbol = match reminder.priority {
                        1 => "!!!",
                        2 => "!!",
                        3 => "!",
                        _ => "",
                    };

                    let mut lines = vec![Line::from(vec![
                        Span::styled(completion_symbol, completion_style),
                        Span::raw(" "),
                        Span::styled(&reminder.title, title_style),
                        if !priority_symbol.is_empty() {
                            Span::styled(
                                format!(" {priority_symbol}"),
                                Style::default().fg(Color::Red),
                            )
                        } else {
                            Span::raw("")
                        },
                    ])];

                    if let Some(notes) = &reminder.notes {
                        if !notes.is_empty() {
                            lines.push(Line::from(format!("  {notes}")));
                        }
                    }

                    if let Some(due_date) = &reminder.due_date {
                        lines.push(Line::from(vec![
                            Span::raw("  Due: "),
                            Span::styled(due_date, Style::default().fg(Color::Yellow)),
                        ]));
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
                        .title(self.list_title.clone()),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("❯ ");

            f.render_stateful_widget(list_widget, main_layout[0], &mut self.list_state);
        }

        // Instructions at the bottom
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Green).bold()),
            Span::raw(" or "),
            Span::styled("↑/↓", Style::default().fg(Color::Green).bold()),
            Span::raw(" to navigate • "),
            Span::styled("Space", Style::default().fg(Color::Green).bold()),
            Span::raw(" to toggle • "),
            Span::styled("Esc", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" to go back • "),
            Span::styled("q", Style::default().fg(Color::Red).bold()),
            Span::raw(" to quit"),
        ])])
        .block(Block::default().borders(Borders::ALL))
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
