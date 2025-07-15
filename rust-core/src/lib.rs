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

#[derive(uniffi::Record, Clone)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub completed: bool,
    pub priority: u8,
    pub due_date: Option<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct NewReminder {
    pub title: String,
    pub notes: Option<String>,
    pub due_date: Option<String>,
    pub list_id: String,
    pub priority: u8,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum TuiAction {
    Quit,
    SelectList { list_id: String },
    ToggleReminder { reminder_id: String },
    DeleteReminder { reminder_id: String },
    CreateReminder { new_reminder: NewReminder },
    Back,
    Refresh,
    ToggleCompletedVisibility,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_completed_visibility_action() {
        // Test that the ToggleCompletedVisibility action is properly defined
        let action = TuiAction::ToggleCompletedVisibility;
        
        match action {
            TuiAction::ToggleCompletedVisibility => {
                // Success - the action exists and matches
                assert!(true);
            }
            _ => {
                panic!("ToggleCompletedVisibility action not found");
            }
        }
    }

    #[test]
    fn test_tui_app_with_reminders() {
        // Create test data
        let lists = vec![
            ReminderList {
                id: "test-list".to_string(),
                name: "Test List".to_string(),
                color: "#FF0000".to_string(),
                count: 3,
            }
        ];
        
        let reminders = vec![
            Reminder {
                id: "rem-1".to_string(),
                title: "Complete task".to_string(),
                notes: Some("This is done".to_string()),
                completed: true,
                priority: 1,
                due_date: None,
            },
            Reminder {
                id: "rem-2".to_string(),
                title: "Incomplete task".to_string(),
                notes: None,
                completed: false,
                priority: 2,
                due_date: None,
            },
            Reminder {
                id: "rem-3".to_string(),
                title: "Another incomplete".to_string(),
                notes: None,
                completed: false,
                priority: 0,
                due_date: None,
            }
        ];

        // Test TUI app creation
        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        
        // Test setting reminders
        tui_app.set_reminders(reminders.clone());
        
        // Test filtering logic
        let filtered_incomplete = tui_app.get_filtered_reminders_for_test();
        
        // Initially show_completed_todos is false, so should only show incomplete
        assert_eq!(filtered_incomplete.len(), 2, "Should show 2 incomplete reminders");
        
        // Toggle visibility to show completed
        tui_app.set_show_completed_todos(true);
        let filtered_all = tui_app.get_filtered_reminders_for_test();
        assert_eq!(filtered_all.len(), 3, "Should show all 3 reminders when toggle is on");
        
        // Toggle back to hide completed
        tui_app.set_show_completed_todos(false);
        let filtered_incomplete_again = tui_app.get_filtered_reminders_for_test();
        assert_eq!(filtered_incomplete_again.len(), 2, "Should show 2 incomplete reminders again");
        
        println!("✅ Toggle completed visibility test passed!");
    }

    #[test]
    fn test_default_hide_completed() {
        // Test that completed todos are hidden by default
        let lists = vec![
            ReminderList {
                id: "test-list".to_string(),
                name: "Test List".to_string(),
                color: "#0000FF".to_string(),
                count: 1,
            }
        ];

        let tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        
        // By default, should hide completed todos
        assert!(!tui_app.show_completed_todos(), "Completed todos should be hidden by default");
        
        println!("✅ Default hide completed test passed!");
    }

    #[test]
    fn test_toggle_with_completed_reminders() {
        // Test toggling visibility with actual completed reminders
        let lists = vec![
            ReminderList {
                id: "test-list".to_string(),
                name: "Test List".to_string(),
                color: "#00FF00".to_string(),
                count: 4,
            }
        ];
        
        let reminders = vec![
            Reminder {
                id: "rem-1".to_string(),
                title: "Completed task 1".to_string(),
                notes: None,
                completed: true,
                priority: 1,
                due_date: None,
            },
            Reminder {
                id: "rem-2".to_string(),
                title: "Incomplete task".to_string(),
                notes: None,
                completed: false,
                priority: 2,
                due_date: None,
            },
            Reminder {
                id: "rem-3".to_string(),
                title: "Completed task 2".to_string(),
                notes: Some("This is done".to_string()),
                completed: true,
                priority: 0,
                due_date: None,
            },
            Reminder {
                id: "rem-4".to_string(),
                title: "Another incomplete".to_string(),
                notes: None,
                completed: false,
                priority: 3,
                due_date: None,
            }
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        tui_app.set_reminders(reminders.clone());
        
        // Initially should show only incomplete (2 reminders)
        let initial_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(initial_filtered.len(), 2, "Should initially show 2 incomplete reminders");
        assert!(initial_filtered.iter().all(|r| !r.completed), "All shown reminders should be incomplete");
        
        // Toggle to show completed
        tui_app.set_show_completed_todos(true);
        let all_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(all_filtered.len(), 4, "Should show all 4 reminders when toggle is on");
        
        // Verify we have both completed and incomplete
        let completed_count = all_filtered.iter().filter(|r| r.completed).count();
        let incomplete_count = all_filtered.iter().filter(|r| !r.completed).count();
        assert_eq!(completed_count, 2, "Should show 2 completed reminders");
        assert_eq!(incomplete_count, 2, "Should show 2 incomplete reminders");
        
        // Toggle back to hide completed
        tui_app.set_show_completed_todos(false);
        let final_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(final_filtered.len(), 2, "Should show 2 incomplete reminders again");
        assert!(final_filtered.iter().all(|r| !r.completed), "All shown reminders should be incomplete again");
        
        println!("✅ Toggle with completed reminders test passed!");
    }
}