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
use std::time::{Duration, Instant};

pub struct TUIApp {
    lists: Vec<ReminderList>,
    current_reminders: Vec<Reminder>,
    current_view: AppView,
    previous_view: Option<AppView>,
    selected_index: usize,
    list_state: ListState,
    actions: Vec<TuiAction>,
    should_exit: bool,
    last_key: Option<KeyCode>,
    last_key_time: Option<Instant>,
    create_form: Option<CreateReminderForm>,
    status_log: Vec<String>,
    is_loading: bool,
    loading_message: String,
    loading_animation_state: usize,
    last_animation_update: Option<Instant>,
    show_completed_todos: bool,
    search_state: SearchState,
    all_reminders: Vec<(Reminder, String)>, // (reminder, list_name) for global search
}

#[derive(Clone, Debug)]
struct SearchState {
    is_active: bool,
    query: String,
    is_global: bool, // true = search all lists, false = search current list
    has_results: bool, // track if we have filtered results to show
    last_escape_time: Option<Instant>, // for double-escape behavior
}

impl SearchState {
    fn new() -> Self {
        Self {
            is_active: false,
            query: String::new(),
            is_global: false,
            has_results: false,
            last_escape_time: None,
        }
    }
    
    fn start_search(&mut self, is_global: bool) {
        self.is_active = true;
        self.is_global = is_global;
        self.query.clear();
        self.has_results = false;
        self.last_escape_time = None;
    }
    
    fn add_char(&mut self, c: char) {
        if self.is_active {
            self.query.push(c);
            self.has_results = !self.query.is_empty();
        }
    }
    
    fn remove_char(&mut self) {
        if self.is_active && !self.query.is_empty() {
            self.query.pop();
            self.has_results = !self.query.is_empty();
        }
    }
    
    fn exit_search(&mut self) {
        self.is_active = false;
        // Keep has_results to maintain filtering
    }
    
    fn clear_search(&mut self) {
        self.is_active = false;
        self.query.clear();
        self.has_results = false;
        self.is_global = false;
        self.last_escape_time = None;
    }
}


#[derive(Clone, Debug)]
pub enum AppView {
    Loading,
    Lists,
    Reminders { list_id: String },
    CreateReminder,
}

#[derive(Clone, Debug)]
struct CreateReminderForm {
    title: String,
    notes: String,
    due_date: String,
    selected_list_id: String,
    priority: u8,
    current_field: usize,
}

impl CreateReminderForm {
    fn new(lists: &[ReminderList], default_list_id: Option<String>) -> Self {
        let selected_list_id = default_list_id
            .unwrap_or_else(|| lists.first().map(|l| l.id.clone()).unwrap_or_default());
        
        Self {
            title: String::new(),
            notes: String::new(),
            due_date: String::new(),
            selected_list_id,
            priority: 0,
            current_field: 0,
        }
    }
}

impl TUIApp {
    pub fn new(lists: Vec<ReminderList>) -> Result<Self, RemError> {
        let mut list_state = ListState::default();
        if !lists.is_empty() {
            list_state.select(Some(0));
        }

        let current_view = if lists.is_empty() { AppView::Loading } else { AppView::Lists };
        
        Ok(Self {
            lists,
            current_reminders: Vec::new(),
            current_view,
            previous_view: None,
            selected_index: 0,
            list_state,
            actions: Vec::new(),
            should_exit: false,
            last_key: None,
            last_key_time: None,
            create_form: None,
            status_log: Vec::new(),
            is_loading: false,
            loading_message: String::new(),
            loading_animation_state: 0,
            last_animation_update: None,
            show_completed_todos: false,
            search_state: SearchState::new(),
            all_reminders: Vec::new(),
        })
    }

    pub fn set_reminders(&mut self, reminders: Vec<Reminder>) {
        self.current_reminders = reminders.clone();
        
        // If we're in global search mode, also populate all_reminders for filtering
        if self.search_state.is_global || self.is_in_global_search_view() {
            // For global search, create all_reminders with a placeholder list name
            // This will be populated correctly when we have proper list names
            self.all_reminders = reminders.into_iter().map(|r| (r, "Unknown List".to_string())).collect();
        }
        
        self.selected_index = 0;
        self.list_state.select(if self.current_reminders.is_empty() { None } else { Some(0) });
    }
    
    pub fn set_reminders_with_global_data(&mut self, reminders: Vec<Reminder>, all_reminders: Vec<(Reminder, String)>) {
        self.current_reminders = reminders;
        self.all_reminders = all_reminders;
        self.selected_index = 0;
        self.list_state.select(if self.current_reminders.is_empty() { None } else { Some(0) });
    }
    
    pub fn set_all_reminders(&mut self, all_reminders: Vec<(Reminder, String)>) {
        self.all_reminders = all_reminders;
    }

    pub fn add_status_log(&mut self, message: String) {
        self.status_log.push(message);
        // Keep only last 5 messages to avoid UI clutter
        if self.status_log.len() > 5 {
            self.status_log.remove(0);
        }
    }

    pub fn set_loading(&mut self, loading: bool, message: String) {
        self.is_loading = loading;
        if loading {
            self.add_status_log(format!("⏳ {}", message));
        }
        self.loading_message = message;
    }

    pub fn set_lists(&mut self, lists: Vec<ReminderList>) {
        self.lists = lists;
        if !self.lists.is_empty() && matches!(self.current_view, AppView::Loading) {
            self.current_view = AppView::Lists;
            self.add_status_log("✅ Lists loaded successfully".to_string());
        }
    }

    fn reset_selection_for_filtered_reminders(&mut self) {
        let filtered_count = self.get_filtered_reminders().len();
        if filtered_count == 0 {
            self.selected_index = 0;
            self.list_state.select(None);
        } else {
            if self.selected_index >= filtered_count {
                self.selected_index = filtered_count - 1;
            }
            self.list_state.select(Some(self.selected_index));
        }
    }

    fn get_filtered_reminders(&self) -> Vec<&Reminder> {
        // For global search, we need to search across all reminders
        let base_reminders: Vec<&Reminder> = if self.search_state.is_global {
            // Use all_reminders for global search (regardless of has_results)
            if self.show_completed_todos {
                self.all_reminders.iter().map(|(r, _)| r).collect()
            } else {
                self.all_reminders.iter().map(|(r, _)| r).filter(|r| !r.completed).collect()
            }
        } else {
            // Use current_reminders for list-specific view
            if self.show_completed_todos {
                self.current_reminders.iter().collect()
            } else {
                self.current_reminders.iter().filter(|r| !r.completed).collect()
            }
        };
        
        let mut reminders = base_reminders;
        
        // Apply search filter if active
        if self.search_state.has_results && !self.search_state.query.is_empty() {
            let query = self.search_state.query.to_lowercase();
            reminders.retain(|reminder| {
                // Search in title and notes
                reminder.title.to_lowercase().contains(&query) ||
                reminder.notes.as_ref().map_or(false, |notes| notes.to_lowercase().contains(&query))
            });
        }
        
        reminders
    }

    // Public method for testing
    pub fn get_filtered_reminders_for_test(&self) -> Vec<&Reminder> {
        self.get_filtered_reminders()
    }

    // Public field access for testing
    pub fn show_completed_todos(&self) -> bool {
        self.show_completed_todos
    }

    pub fn set_show_completed_todos(&mut self, show: bool) {
        self.show_completed_todos = show;
    }

    // Search-related public methods for testing
    pub fn is_search_active(&self) -> bool {
        self.search_state.is_active
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_state.query
    }

    pub fn start_global_search(&mut self) {
        self.search_state.start_search(true);
    }

    pub fn start_list_search(&mut self) {
        self.search_state.start_search(false);
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_state.add_char(c);
    }

    pub fn remove_search_char(&mut self) {
        self.search_state.remove_char();
    }

    pub fn clear_search(&mut self) {
        self.search_state.clear_search();
    }

    pub fn exit_search(&mut self) {
        self.search_state.exit_search();
    }

    pub fn is_global_search(&self) -> bool {
        self.search_state.is_global
    }

    pub fn has_search_results(&self) -> bool {
        self.search_state.has_results
    }

    // Additional test helper methods
    pub fn get_current_view(&self) -> &AppView {
        &self.current_view
    }

    pub fn set_current_view(&mut self, view: AppView) {
        self.current_view = view;
    }

    // Helper method to get list name for a reminder in global search
    pub fn get_list_name_for_reminder(&self, reminder_id: &str) -> Option<&str> {
        self.all_reminders
            .iter()
            .find(|(r, _)| r.id == reminder_id)
            .map(|(_, list_name)| list_name.as_str())
    }

    // Check if we're currently in global search mode
    pub fn is_in_global_search_view(&self) -> bool {
        matches!(&self.current_view, AppView::Reminders { list_id } if list_id == "global")
    }

    pub fn run(&mut self) -> Result<Vec<TuiAction>, RemError> {
        // Setup terminal with better error handling
        enable_raw_mode().map_err(|e| {
            RemError::TUIError { 
                message: format!("Failed to enable raw mode: {}. Try running in a different terminal.", e) 
            }
        })?;
        
        let mut stdout = io::stdout();
        
        // Try alternate screen and mouse capture with fallback
        if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            // Fallback: try without mouse capture
            execute!(stdout, EnterAlternateScreen)
                .map_err(|e2| RemError::TUIError { 
                    message: format!("Terminal setup failed: {}. Original error: {}", e2, e) 
                })?;
        }
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| {
            RemError::TUIError { 
                message: format!("Failed to create terminal: {}. Check terminal compatibility.", e) 
            }
        })?;

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
        // Handle search mode first
        if self.search_state.is_active {
            self.handle_search_key_event(key);
            return;
        }
        
        // Special handling for global search view when search is not active
        if self.is_in_global_search_view() && !self.search_state.is_active {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.search_state.clear_search();
                    self.current_view = AppView::Lists;
                    self.actions.push(TuiAction::Back);
                    self.add_status_log("🔍 Global search closed".to_string());
                    return;
                }
                KeyCode::Char('/') => {
                    self.search_state.start_search(true); // Re-activate global search
                    self.add_status_log("🔍 Global search reactivated...".to_string());
                    return;
                }
                _ => {
                    // For other keys, fall through to normal reminders handling
                }
            }
        }
        
        // Handle search activation
        if key.code == KeyCode::Char('/') {
            match &self.current_view {
                AppView::Lists => {
                    self.search_state.start_search(true); // Global search from lists
                    self.add_status_log("🔍 Global search started...".to_string());
                    // Switch to reminders view with special global identifier
                    self.current_view = AppView::Reminders { list_id: "global".to_string() };
                    // Trigger loading of all reminders
                    self.actions.push(TuiAction::GlobalSearch { query: "".to_string() });
                    return;
                }
                AppView::Reminders { .. } => {
                    self.search_state.start_search(false); // List-specific search
                    self.add_status_log("🔍 List search started...".to_string());
                    return;
                }
                _ => {}
            }
        }
        
        // Handle normal view logic
        match &self.current_view {
            AppView::Loading => {
                // Only allow quit during loading
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    self.actions.push(TuiAction::Quit);
                    self.should_exit = true;
                }
            }
            AppView::Lists => self.handle_lists_key_event(key),
            AppView::Reminders { list_id } => self.handle_reminders_key_event(key, list_id.clone()),
            AppView::CreateReminder => self.handle_create_reminder_key_event(key),
        }
        
        // Update last key for sequence tracking with timing
        self.last_key = Some(key.code);
        self.last_key_time = Some(Instant::now());
    }

    fn handle_search_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let now = Instant::now();
                if let Some(last_escape) = self.search_state.last_escape_time {
                    // Double escape within 1 second = clear search completely
                    if now.duration_since(last_escape) < Duration::from_millis(1000) {
                        self.search_state.clear_search();
                        self.reset_selection_for_filtered_reminders();
                        
                        // For global search, go back to Lists view
                        if self.is_in_global_search_view() {
                            self.current_view = AppView::Lists;
                            self.actions.push(TuiAction::Back);
                            self.add_status_log("🔍 Global search closed".to_string());
                        } else {
                            self.add_status_log("🔍 Search cleared".to_string());
                        }
                        return;
                    }
                }
                // Single escape = exit search mode but keep results
                self.search_state.exit_search();
                self.search_state.last_escape_time = Some(now);
                
                // For global search, first escape goes back to Lists view
                if self.is_in_global_search_view() {
                    self.current_view = AppView::Lists;
                    self.actions.push(TuiAction::Back);
                    self.add_status_log("🔍 Global search closed".to_string());
                } else {
                    self.add_status_log("🔍 Search mode exited (ESC again to clear)".to_string());
                }
            }
            KeyCode::Char(c) => {
                self.search_state.add_char(c);
                self.update_search_results();
                self.reset_selection_for_filtered_reminders();
            }
            KeyCode::Backspace => {
                self.search_state.remove_char();
                self.update_search_results();
                self.reset_selection_for_filtered_reminders();
            }
            KeyCode::Enter => {
                // Enter confirms search and exits search mode
                self.search_state.exit_search();
                let query = &self.search_state.query;
                if !query.is_empty() {
                    self.add_status_log(format!("🔍 Search for '{}' applied", query));
                } else {
                    self.search_state.clear_search();
                    self.add_status_log("🔍 Search cleared".to_string());
                }
                self.reset_selection_for_filtered_reminders();
            }
            _ => {
                // Ignore other keys in search mode
            }
        }
    }

    fn update_search_results(&mut self) {
        let query = &self.search_state.query;
        self.search_state.has_results = !query.is_empty();
        
        if !query.is_empty() {
            let search_type = if self.search_state.is_global { "global" } else { "list" };
            self.add_status_log(format!("🔍 {} search: '{}'", search_type, query));
        }
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
                    let list_id = list.id.clone();
                    // Switch view immediately
                    self.current_view = AppView::Reminders { list_id: list_id.clone() };
                    self.add_status_log("📋 Loading reminders...".to_string());
                    self.actions.push(TuiAction::SelectList { list_id });
                }
            }
            KeyCode::Char('c') => {
                let default_list_id = if !self.lists.is_empty() {
                    Some(self.lists[self.selected_index].id.clone())
                } else {
                    None
                };
                self.previous_view = Some(self.current_view.clone());
                self.create_form = Some(CreateReminderForm::new(&self.lists, default_list_id.clone()));
                self.current_view = AppView::CreateReminder;
            }
            KeyCode::Char('h') => {
                self.show_completed_todos = !self.show_completed_todos;
                let status = if self.show_completed_todos { "shown" } else { "hidden" };
                self.add_status_log(format!("👁️ Completed todos {}", status));
                // Note: We don't push the action here as this is for lists view
                // The action would cause the app to exit this view
            }
            _ => {}
        }
    }

    fn handle_reminders_key_event(&mut self, key: crossterm::event::KeyEvent, list_id: String) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // For global search, clear search state when going back
                if list_id == "global" {
                    self.search_state.clear_search();
                    self.add_status_log("🔍 Global search closed".to_string());
                }
                self.actions.push(TuiAction::Back);
                self.current_view = AppView::Lists;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let filtered_reminders = self.get_filtered_reminders();
                if !filtered_reminders.is_empty() {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = filtered_reminders.len() - 1;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let filtered_reminders = self.get_filtered_reminders();
                if !filtered_reminders.is_empty() {
                    if self.selected_index < filtered_reminders.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                    self.list_state.select(Some(self.selected_index));
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let filtered_reminders = self.get_filtered_reminders();
                if let Some(reminder) = filtered_reminders.get(self.selected_index) {
                    let reminder_id = reminder.id.clone();
                    self.add_status_log("✅ Toggling reminder...".to_string());
                    self.actions.push(TuiAction::ToggleReminder { reminder_id });
                }
            }
            KeyCode::Char('d') => {
                // Check if this is the second 'd' for 'dd' delete command
                let is_dd_sequence = if let (Some(KeyCode::Char('d')), Some(last_time)) = (self.last_key, self.last_key_time) {
                    // Allow up to 1000ms between 'd' presses
                    last_time.elapsed() < Duration::from_millis(1000)
                } else {
                    false
                };

                if is_dd_sequence {
                    let filtered_reminders = self.get_filtered_reminders();
                    if let Some(reminder) = filtered_reminders.get(self.selected_index) {
                        self.actions.push(TuiAction::DeleteReminder { reminder_id: reminder.id.clone() });
                    }
                }
                // Note: last_key will be updated after this function returns
            }
            KeyCode::Delete => {
                // Alternative: Use Delete key for immediate deletion (no sequence needed)
                let filtered_reminders = self.get_filtered_reminders();
                if let Some(reminder) = filtered_reminders.get(self.selected_index) {
                    self.actions.push(TuiAction::DeleteReminder { reminder_id: reminder.id.clone() });
                }
            }
            KeyCode::Char('c') => {
                self.previous_view = Some(self.current_view.clone());
                self.create_form = Some(CreateReminderForm::new(&self.lists, Some(list_id.clone())));
                self.current_view = AppView::CreateReminder;
            }
            KeyCode::Char('h') => {
                self.show_completed_todos = !self.show_completed_todos;
                let status = if self.show_completed_todos { "shown" } else { "hidden" };
                self.add_status_log(format!("👁️ Completed todos {}", status));
                // Reset selection to ensure we stay within filtered bounds
                self.reset_selection_for_filtered_reminders();
                // Don't push action - handle entirely within TUI for immediate re-render
            }
            _ => {}
        }
    }

    fn handle_create_reminder_key_event(&mut self, key: crossterm::event::KeyEvent) {
        if let Some(ref mut form) = self.create_form {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.actions.push(TuiAction::Back);
                    // Return to previous view or Lists as fallback
                    self.current_view = self.previous_view.take().unwrap_or(AppView::Lists);
                    self.create_form = None;
                }
                KeyCode::Tab => {
                    form.current_field = (form.current_field + 1) % 5; // 5 fields: title, notes, date, list, priority
                }
                KeyCode::BackTab => {
                    form.current_field = if form.current_field == 0 { 4 } else { form.current_field - 1 };
                }
                KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                    // Ctrl+S to save/submit
                    if !form.title.trim().is_empty() {
                        let new_reminder = crate::NewReminder {
                            title: form.title.clone(),
                            notes: if form.notes.trim().is_empty() { None } else { Some(form.notes.clone()) },
                            due_date: if form.due_date.trim().is_empty() { None } else { Some(form.due_date.clone()) },
                            list_id: form.selected_list_id.clone(),
                            priority: form.priority,
                        };
                        self.actions.push(TuiAction::CreateReminder { new_reminder });
                        self.create_form = None;
                        // Return to previous view or Lists as fallback
                        self.current_view = self.previous_view.take().unwrap_or(AppView::Lists);
                    }
                }
                KeyCode::Char(c) => {
                    match form.current_field {
                        0 => form.title.push(c), // Title field
                        1 => form.notes.push(c), // Notes field
                        2 => form.due_date.push(c), // Date field
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    match form.current_field {
                        0 => { form.title.pop(); }
                        1 => { form.notes.pop(); }
                        2 => { form.due_date.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Up | KeyCode::Down => {
                    match form.current_field {
                        3 => { // List field
                            if key.code == KeyCode::Up {
                                if let Some(current_idx) = self.lists.iter().position(|l| l.id == form.selected_list_id) {
                                    let new_idx = if current_idx == 0 { self.lists.len() - 1 } else { current_idx - 1 };
                                    form.selected_list_id = self.lists[new_idx].id.clone();
                                }
                            } else {
                                if let Some(current_idx) = self.lists.iter().position(|l| l.id == form.selected_list_id) {
                                    let new_idx = (current_idx + 1) % self.lists.len();
                                    form.selected_list_id = self.lists[new_idx].id.clone();
                                }
                            }
                        }
                        4 => { // Priority field
                            if key.code == KeyCode::Up && form.priority < 9 {
                                form.priority += 1;
                            } else if key.code == KeyCode::Down && form.priority > 0 {
                                form.priority -= 1;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        match &self.current_view {
            AppView::Loading => self.render_loading(f),
            AppView::Lists => self.render_lists(f),
            AppView::Reminders { .. } => self.render_reminders(f),
            AppView::CreateReminder => self.render_create_reminder(f),
        }
    }

    fn render_loading(&mut self, f: &mut Frame) {
        // Update animation state every 150ms (similar to Claude Code timing)
        let now = Instant::now();
        if let Some(last_update) = self.last_animation_update {
            if now.duration_since(last_update) >= Duration::from_millis(150) {
                self.loading_animation_state = (self.loading_animation_state + 1) % 8;
                self.last_animation_update = Some(now);
            }
        } else {
            self.last_animation_update = Some(now);
        }

        // Claude Code style thinking animation sequence
        let thinking_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        let current_char = thinking_chars[self.loading_animation_state];
        
        let area = f.area();
        
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Loading content
                Constraint::Length(4),  // Controls
                Constraint::Length(3)   // Status log
            ])
            .margin(1)
            .split(area);

        let loading_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    current_char,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled(
                    " Loading Apple Reminders...",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                &self.loading_message,
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
            .alignment(Alignment::Center);

        f.render_widget(paragraph, main_layout[0]);

        // Loading controls
        let instructions = Paragraph::new(vec![Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
            Span::styled(" quit  ", Style::default().fg(Color::Gray)),
            Span::styled("⏳", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" loading...", Style::default().fg(Color::Gray)),
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
        
        // Status log
        self.render_status_log(f, main_layout[2]);
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
            .constraints([
                Constraint::Min(0),     // List content
                Constraint::Length(4),  // Controls
                Constraint::Length(3)   // Status log
            ])
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
        let visibility_text = if self.show_completed_todos { "hide completed" } else { "show completed" };
        let visibility_display = format!(" {}  ", visibility_text);
        let instructions = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
                Span::styled("⏎", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" select  ", Style::default().fg(Color::Gray)),
                Span::styled("c", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" create", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(visibility_display, Style::default().fg(Color::Gray)),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" quit", Style::default().fg(Color::Gray)),
            ]),
        ])
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
        
        // Status log
        self.render_status_log(f, main_layout[2]);
    }

    fn render_reminders(&mut self, f: &mut Frame) {
        let area = f.area();
        
        let filtered_reminders: Vec<Reminder> = self.get_filtered_reminders().into_iter().cloned().collect();
        
        if filtered_reminders.is_empty() {
            let message = if self.current_reminders.is_empty() {
                "📭 No reminders in this list"
            } else {
                "📭 No incomplete reminders (press 'h' to show completed)"
            };
            
            let empty_text = vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    message,
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
                            if self.is_in_global_search_view() { " 🔍 Global Search " } else { " 📝 Reminders " },
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

        let main_layout = if self.search_state.is_active || self.search_state.has_results {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Search bar
                    Constraint::Min(0),     // Reminders content
                    Constraint::Length(4),  // Controls
                    Constraint::Length(3)   // Status log
                ])
                .margin(1)
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),     // Reminders content
                    Constraint::Length(4),  // Controls
                    Constraint::Length(3)   // Status log
                ])
                .margin(1)
                .split(area)
        };

        // Render search bar if active or has results
        let content_index = if self.search_state.is_active || self.search_state.has_results {
            self.render_search_bar(f, main_layout[0]);
            1 // Content area index when search bar is shown
        } else {
            0 // Content area index when no search bar
        };

        // Create reminder items
        let items: Vec<ListItem> = filtered_reminders
            .iter()
            .enumerate()
            .map(|(i, reminder)| {
                let is_selected = i == self.selected_index;
                
                let checkbox = if reminder.completed { "☑" } else { "☐" };
                let title_color = if reminder.completed {
                    if is_selected { Color::LightBlue } else { Color::Gray }
                } else {
                    if is_selected { Color::White } else { Color::White }
                };
                let title_modifier = if reminder.completed { Modifier::CROSSED_OUT } else { Modifier::empty() };

                // Build the title line with optional list name for global search
                let mut title_spans = vec![
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
                ];

                // Add list name for global search
                if self.is_in_global_search_view() {
                    if let Some(list_name) = self.get_list_name_for_reminder(&reminder.id) {
                        title_spans.push(Span::styled(
                            format!("[{}] ",list_name),
                            Style::default()
                                .fg(Color::LightBlue)
                                .add_modifier(Modifier::BOLD)
                        ));
                    }
                }

                title_spans.push(Span::styled(
                    &reminder.title,
                    Style::default()
                        .fg(title_color)
                        .add_modifier(title_modifier | if is_selected { Modifier::UNDERLINED } else { Modifier::empty() })
                ));

                let mut lines = vec![Line::from(title_spans)];

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

                if i < filtered_reminders.len() - 1 {
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
                        if self.is_in_global_search_view() { " 🔍 Global Search " } else { " 📝 Reminders " },
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

        f.render_stateful_widget(list_widget, main_layout[content_index], &mut self.list_state);

        // Instructions
        let visibility_text = if self.show_completed_todos { "hide completed" } else { "show completed" };
        let visibility_display = format!(" {}  ", visibility_text);
        
        // Different instructions for global search
        let instructions = if self.is_in_global_search_view() {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" or ", Style::default().fg(Color::Gray)),
                    Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
                    Span::styled("⏎/space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" toggle  ", Style::default().fg(Color::Gray)),
                    Span::styled("dd/Del", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" delete", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(visibility_display, Style::default().fg(Color::Gray)),
                    Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" or ", Style::default().fg(Color::Gray)),
                    Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" back to lists", Style::default().fg(Color::Gray)),
                ]),
            ];
            
            // Add search instructions if search is active
            if self.search_state.is_active {
                lines.push(Line::from(vec![
                    Span::styled("Type", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" to search  ", Style::default().fg(Color::Gray)),
                    Span::styled("Backspace", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" to delete", Style::default().fg(Color::Gray)),
                ]));
            }
            
            Paragraph::new(lines)
        } else {
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" or ", Style::default().fg(Color::Gray)),
                    Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
                    Span::styled("⏎/space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" toggle  ", Style::default().fg(Color::Gray)),
                    Span::styled("dd/Del", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" delete", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                    Span::styled("c", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" create  ", Style::default().fg(Color::Gray)),
                    Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(visibility_display, Style::default().fg(Color::Gray)),
                    Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                    Span::styled(" back", Style::default().fg(Color::Gray)),
                ]),
            ])
        }
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

        f.render_widget(instructions, main_layout[content_index + 1]);
        
        // Status log
        self.render_status_log(f, main_layout[content_index + 2]);
    }

    fn render_search_bar(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let search_type = if self.search_state.is_global { "Global" } else { "List" };
        let title = format!(" 🔍 {} Search ", search_type);
        
        let search_text = if self.search_state.is_active {
            format!("{}_", self.search_state.query) // Show cursor with underscore
        } else {
            self.search_state.query.clone()
        };
        
        let placeholder = if search_text.is_empty() && !self.search_state.is_active {
            if self.search_state.is_global {
                "Press '/' to search all reminders..."
            } else {
                "Press '/' to search this list..."
            }
        } else {
            ""
        };
        
        let display_text = if !placeholder.is_empty() {
            placeholder
        } else {
            &search_text
        };
        
        let text_color = if self.search_state.is_active {
            Color::Yellow
        } else if !self.search_state.query.is_empty() {
            Color::Green
        } else {
            Color::Gray
        };
        
        let border_color = if self.search_state.is_active {
            Color::Yellow
        } else {
            Color::Blue
        };
        
        let search_paragraph = Paragraph::new(display_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        title,
                        Style::default().fg(border_color).add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Left)
                    .style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(text_color))
            .alignment(Alignment::Left);
        
        f.render_widget(search_paragraph, area);
    }

    fn render_create_reminder(&mut self, f: &mut Frame) {
        let area = f.area();
        
        if let Some(ref form) = self.create_form {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),     // Form content
                    Constraint::Length(3),  // Controls
                    Constraint::Length(3)   // Status log
                ])
                .margin(2)
                .split(area);

            // Form fields layout
            let form_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Length(5), // Notes
                    Constraint::Length(3), // Date
                    Constraint::Length(3), // List
                    Constraint::Length(3), // Priority
                ])
                .split(main_layout[0]);

            // Title field
            let title_style = if form.current_field == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let title_paragraph = Paragraph::new(if form.title.is_empty() { "New Reminder" } else { &form.title })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" Title ", title_style))
                        .style(title_style)
                );
            f.render_widget(title_paragraph, form_layout[0]);

            // Notes field
            let notes_style = if form.current_field == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let notes_paragraph = Paragraph::new(if form.notes.is_empty() { "Add some notes..." } else { &form.notes })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" Notes ", notes_style))
                        .style(notes_style)
                )
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(notes_paragraph, form_layout[1]);

            // Date field
            let date_style = if form.current_field == 2 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let date_paragraph = Paragraph::new(if form.due_date.is_empty() { "No Date" } else { &form.due_date })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" Date ", date_style))
                        .style(date_style)
                );
            f.render_widget(date_paragraph, form_layout[2]);

            // List field
            let list_style = if form.current_field == 3 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let selected_list_name = self.lists.iter()
                .find(|l| l.id == form.selected_list_id)
                .map(|l| l.name.as_str())
                .unwrap_or("Unknown");
                
            let list_paragraph = Paragraph::new(selected_list_name)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" List ", list_style))
                        .style(list_style)
                );
            f.render_widget(list_paragraph, form_layout[3]);

            // Priority field
            let priority_style = if form.current_field == 4 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let priority_text = if form.priority == 0 { "None".to_string() } else { form.priority.to_string() };
            let priority_paragraph = Paragraph::new(priority_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(Span::styled(" Priority ", priority_style))
                        .style(priority_style)
                );
            f.render_widget(priority_paragraph, form_layout[4]);

            // Instructions
            let instructions = Paragraph::new(vec![Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" navigate  ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" create  ", Style::default().fg(Color::Gray)),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
                Span::styled(" cancel", Style::default().fg(Color::Gray)),
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
            
            // Status log
            self.render_status_log(f, main_layout[2]);
        }
    }

    fn render_status_log(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let log_lines: Vec<Line> = if self.status_log.is_empty() {
            vec![Line::from(Span::styled(
                "Ready",
                Style::default().fg(Color::Green)
            ))]
        } else {
            self.status_log.iter().map(|msg| {
                // Add thinking animation to loading messages
                if msg.contains("Loading") || msg.contains("⏳") {
                    let thinking_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                    let current_char = thinking_chars[self.loading_animation_state];
                    Line::from(vec![
                        Span::styled(
                            current_char,
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        ),
                        Span::styled(
                            format!(" {}", msg.replace("⏳ ", "")),
                            Style::default().fg(Color::Cyan)
                        )
                    ])
                } else {
                    Line::from(Span::styled(
                        msg,
                        Style::default().fg(Color::Cyan)
                    ))
                }
            }).collect()
        };

        let status_paragraph = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Status ",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    ))
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan))
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(status_paragraph, area);
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