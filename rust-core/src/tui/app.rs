use crate::{ReminderList, Reminder, TuiAction, RemError};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Padding},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

pub struct TUIApp {
    lists: Vec<ReminderList>,
    current_reminders: Vec<Reminder>,
    current_view: AppView,
    selected_index: usize,
    list_state: ListState,
    actions: Vec<TuiAction>,
    should_exit: bool,
    last_key: Option<KeyCode>,
}

#[derive(Clone, Debug)]
enum AppView {
    Lists,
    Reminders { list_id: String },
}

impl TUIApp {
    pub fn new(lists: Vec<ReminderList>) -> Result<Self, RemError> {
        let mut list_state = ListState::default();
        if !lists.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            lists,
            current_reminders: Vec::new(),
            current_view: AppView::Lists,
            selected_index: 0,
            list_state,
            actions: Vec::new(),
            should_exit: false,
            last_key: None,
        })
    }

    pub fn set_reminders(&mut self, reminders: Vec<Reminder>) {
        self.current_reminders = reminders;
        self.selected_index = 0;
        self.list_state.select(if self.current_reminders.is_empty() { None } else { Some(0) });
    }

    pub fn run(&mut self) -> Result<Vec<TuiAction>, RemError> {
        // Setup terminal
        enable_raw_mode().map_err(|e| RemError::TUIError { message: e.to_string() })?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| RemError::TUIError { message: e.to_string() })?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| RemError::TUIError { message: e.to_string() })?;

        let result = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode().map_err(|e| RemError::TUIError { message: e.to_string() })?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| RemError::TUIError { message: e.to_string() })?;
        terminal.show_cursor().map_err(|e| RemError::TUIError { message: e.to_string() })?;

        result
    }


    pub fn run_reminders_view(&mut self) -> Result<Vec<TuiAction>, RemError> {
        // For reminders view, just handle the reminders display
        // This is called when we're already in the TUI and switching to reminders
        
        // Setup terminal
        enable_raw_mode().map_err(|e| RemError::TUIError { message: e.to_string() })?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| RemError::TUIError { message: e.to_string() })?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| RemError::TUIError { message: e.to_string() })?;

        let result = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode().map_err(|e| RemError::TUIError { message: e.to_string() })?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| RemError::TUIError { message: e.to_string() })?;
        terminal.show_cursor().map_err(|e| RemError::TUIError { message: e.to_string() })?;

        result
    }

    fn run_app<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<Vec<TuiAction>, RemError> {
        self.actions.clear();
        
        loop {
            terminal.draw(|f| self.ui(f)).map_err(|e| RemError::TUIError { message: e.to_string() })?;

            if self.should_exit {
                break;
            }

            if event::poll(Duration::from_millis(50)).map_err(|e| RemError::TUIError { message: e.to_string() })? {
                if let Event::Key(key) = event::read().map_err(|e| RemError::TUIError { message: e.to_string() })? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key);
                    }
                }
            }

            if !self.actions.is_empty() {
                break;
            }
        }

        Ok(self.actions.clone())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match &self.current_view {
            AppView::Lists => self.handle_lists_key_event(key),
            AppView::Reminders { list_id } => self.handle_reminders_key_event(key, list_id.clone()),
        }
        
        // Update last key for sequence tracking
        self.last_key = Some(key.code);
    }

    fn handle_lists_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.actions.push(TuiAction::Quit);
                self.should_exit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.lists.is_empty() {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.lists.len() - 1;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.lists.is_empty() {
                    if self.selected_index < self.lists.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Enter => {
                if let Some(list) = self.lists.get(self.selected_index) {
                    self.actions.push(TuiAction::SelectList { list_id: list.id.clone() });
                    self.current_view = AppView::Reminders { list_id: list.id.clone() };
                }
            }
            _ => {}
        }
    }

    fn handle_reminders_key_event(&mut self, key: crossterm::event::KeyEvent, _list_id: String) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.actions.push(TuiAction::Back);
                self.current_view = AppView::Lists;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.current_reminders.is_empty() {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.current_reminders.len() - 1;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.current_reminders.is_empty() {
                    if self.selected_index < self.current_reminders.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(reminder) = self.current_reminders.get(self.selected_index) {
                    self.actions.push(TuiAction::ToggleReminder { reminder_id: reminder.id.clone() });
                }
            }
            KeyCode::Char('d') => {
                // Check if this is the second 'd' for 'dd' delete command
                if let Some(KeyCode::Char('d')) = self.last_key {
                    if let Some(reminder) = self.current_reminders.get(self.selected_index) {
                        self.actions.push(TuiAction::DeleteReminder { reminder_id: reminder.id.clone() });
                    }
                }
                // Note: last_key will be updated after this function returns
            }
            _ => {}
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        match &self.current_view {
            AppView::Lists => self.render_lists(f),
            AppView::Reminders { .. } => self.render_reminders(f),
        }
    }

    fn render_lists(&mut self, f: &mut Frame) {
        let area = f.area();
        
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
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .margin(1)
            .split(area);

        // Create list items
        let items: Vec<ListItem> = self
            .lists
            .iter()
            .enumerate()
            .map(|(i, list)| {
                let is_selected = i == self.selected_index;
                let color = parse_color(&list.color);
                
                let count_text = if list.count == 0 {
                    "Empty".to_string()
                } else if list.count == 1 {
                    "1 reminder".to_string()
                } else {
                    format!("{} reminders", list.count)
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
                            &list.name,
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
                            Style::default().fg(if list.count == 0 {
                                Color::DarkGray
                            } else if is_selected {
                                Color::Gray
                            } else {
                                Color::Gray
                            })
                        ),
                    ]),
                ];

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

        // Instructions
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

    fn render_reminders(&mut self, f: &mut Frame) {
        let area = f.area();
        
        if self.current_reminders.is_empty() {
            let empty_text = vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "📭 No reminders in this list",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'q' to go back",
                    Style::default().fg(Color::Gray)
                )),
            ];

            let paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(
                            " 📝 Reminders ",
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD)
                        ))
                        .title_alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Blue))
                )
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .margin(1)
            .split(area);

        // Create reminder items
        let items: Vec<ListItem> = self
            .current_reminders
            .iter()
            .enumerate()
            .map(|(i, reminder)| {
                let is_selected = i == self.selected_index;
                
                let checkbox = if reminder.completed { "☑" } else { "☐" };
                let title_color = if reminder.completed { Color::DarkGray } else { Color::White };
                let title_modifier = if reminder.completed { Modifier::CROSSED_OUT } else { Modifier::empty() };

                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            if is_selected { "▶ " } else { "  " },
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        ),
                        Span::styled(
                            checkbox,
                            Style::default()
                                .fg(if reminder.completed { Color::Green } else { Color::Gray })
                                .add_modifier(Modifier::BOLD)
                        ),
                        Span::raw("  "),
                        Span::styled(
                            &reminder.title,
                            Style::default()
                                .fg(title_color)
                                .add_modifier(title_modifier | if is_selected { Modifier::UNDERLINED } else { Modifier::empty() })
                        ),
                    ]),
                ];

                if let Some(notes) = &reminder.notes {
                    if !notes.is_empty() {
                        lines.push(Line::from(vec![
                            Span::raw("      "),
                            Span::styled(
                                notes,
                                Style::default().fg(Color::DarkGray)
                            ),
                        ]));
                    }
                }

                if i < self.current_reminders.len() - 1 {
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
                        " 📝 Reminders ",
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

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("⏎/space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" toggle  ", Style::default().fg(Color::Gray)),
            Span::styled("dd", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" delete  ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" back", Style::default().fg(Color::Gray)),
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

fn parse_color(color_str: &str) -> Color {
    match color_str {
        s if s.starts_with("#") => {
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