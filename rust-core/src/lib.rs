use std::sync::Mutex;

pub mod tui;
pub mod types;

use tui::TUIApp;

// Global TUI state management
static TUI_APP: Mutex<Option<TUIApp>> = Mutex::new(None);

#[derive(uniffi::Record)]
pub struct ReminderList {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: u32,
}

#[derive(uniffi::Record)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub completed: bool,
    pub priority: u8,
    pub due_date: Option<String>,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum TuiAction {
    Quit,
    SelectList { list_id: String },
    ToggleReminder { reminder_id: String },
    DeleteReminder { reminder_id: String },
    Back,
    Refresh,
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum RemError {
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Data access error: {message}")]
    DataAccessError { message: String },
    #[error("TUI error: {message}")]
    TUIError { message: String },
}

#[uniffi::export]
pub fn start_tui(lists: Vec<ReminderList>) -> Result<Vec<TuiAction>, RemError> {
    let mut tui_app = TUIApp::new(lists)?;
    let actions = tui_app.run()?;
    
    // Store the app state for subsequent calls
    let mut global_tui = TUI_APP.lock().unwrap();
    *global_tui = Some(tui_app);
    
    Ok(actions)
}

#[uniffi::export]
pub fn render_reminders_view(reminders: Vec<Reminder>) -> Result<Vec<TuiAction>, RemError> {
    let mut global_tui = TUI_APP.lock().unwrap();
    
    if let Some(ref mut tui_app) = global_tui.as_mut() {
        tui_app.set_reminders(reminders);
        let actions = tui_app.run_reminders_view()?;
        Ok(actions)
    } else {
        Err(RemError::TUIError { message: "TUI not initialized".to_string() })
    }
}

#[uniffi::export]
pub fn set_reminders(reminders: Vec<Reminder>) -> Result<(), RemError> {
    let mut global_tui = TUI_APP.lock().unwrap();
    
    if let Some(ref mut tui_app) = global_tui.as_mut() {
        tui_app.set_reminders(reminders);
        Ok(())
    } else {
        Err(RemError::TUIError { message: "TUI not initialized".to_string() })
    }
}

uniffi::setup_scaffolding!();