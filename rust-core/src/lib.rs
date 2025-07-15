use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Mutex;

pub mod tui;
pub mod types;

use tui::TUIApp;

// Global TUI state management
struct GlobalTuiState {
    app: TUIApp,
    is_initialized: bool,
}

static TUI_STATE: Mutex<Option<GlobalTuiState>> = Mutex::new(None);

#[derive(uniffi::Record)]
pub struct ReminderList {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: u32,
}

#[derive(uniffi::Record, Clone, Debug)]
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
    GlobalSearch { query: String },
    ShowLoading { message: String },
    DataLoaded,
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
    let mut global_state = TUI_STATE.lock().unwrap();
    *global_state = Some(GlobalTuiState {
        app: tui_app,
        is_initialized: true,
    });

    Ok(actions)
}

#[uniffi::export]
pub fn run_persistent_tui(lists: Vec<ReminderList>) -> Result<Vec<TuiAction>, RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    // Initialize the TUI app
    let mut tui_app = TUIApp::new(lists)?;

    // Setup terminal
    enable_raw_mode().map_err(|e| RemError::TUIError {
        message: format!("Failed to enable raw mode: {e}. Try running in a different terminal."),
    })?;

    let mut stdout = io::stdout();

    // Try alternate screen and mouse capture with fallback
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        // Fallback: try without mouse capture
        execute!(stdout, EnterAlternateScreen).map_err(|e2| RemError::TUIError {
            message: format!("Terminal setup failed: {e2}. Original error: {e}"),
        })?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| RemError::TUIError {
        message: format!("Failed to create terminal: {e}. Check terminal compatibility."),
    })?;

    // Run the first iteration to get initial actions
    let actions = tui_app.run_persistent_iteration(&mut terminal)?;

    // Store the app state globally
    *global_state = Some(GlobalTuiState {
        app: tui_app,
        is_initialized: true,
    });

    Ok(actions)
}

#[uniffi::export]
pub fn continue_persistent_tui() -> Result<Vec<TuiAction>, RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    if let Some(ref mut state) = global_state.as_mut() {
        if !state.is_initialized {
            return Err(RemError::TUIError {
                message: "TUI not properly initialized".to_string(),
            });
        }

        // Create a new terminal for this iteration
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| RemError::TUIError {
            message: format!("Failed to create terminal: {e}. Check terminal compatibility."),
        })?;

        let actions = state.app.run_persistent_iteration(&mut terminal)?;
        Ok(actions)
    } else {
        Err(RemError::TUIError {
            message: "TUI not initialized".to_string(),
        })
    }
}

#[uniffi::export]
pub fn shutdown_tui() -> Result<(), RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    if let Some(mut state) = global_state.take() {
        // Restore terminal
        disable_raw_mode().map_err(|e| RemError::TUIError {
            message: e.to_string(),
        })?;

        // Create a terminal just to properly restore it
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).map_err(|e| {
            RemError::TUIError {
                message: e.to_string(),
            }
        })?;

        // Don't need to call show_cursor since we're exiting anyway

        state.is_initialized = false;
    }

    Ok(())
}

#[uniffi::export]
pub fn render_reminders_view(reminders: Vec<Reminder>) -> Result<Vec<TuiAction>, RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    if let Some(ref mut state) = global_state.as_mut() {
        state.app.set_reminders(reminders);

        // Create a new terminal for this iteration
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| RemError::TUIError {
            message: format!("Failed to create terminal: {e}. Check terminal compatibility."),
        })?;

        let actions = state.app.run_persistent_iteration(&mut terminal)?;
        Ok(actions)
    } else {
        Err(RemError::TUIError {
            message: "TUI not initialized".to_string(),
        })
    }
}

#[uniffi::export]
pub fn set_reminders(reminders: Vec<Reminder>) -> Result<(), RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    if let Some(ref mut state) = global_state.as_mut() {
        // Add status message before setting reminders
        state
            .app
            .add_status_log("✅ Reminders loaded successfully".to_string());
        state.app.set_reminders(reminders);
        Ok(())
    } else {
        Err(RemError::TUIError {
            message: "TUI not initialized".to_string(),
        })
    }
}

#[uniffi::export]
pub fn set_global_reminders(
    reminders: Vec<Reminder>,
    list_names: Vec<String>,
) -> Result<(), RemError> {
    let mut global_state = TUI_STATE.lock().unwrap();

    if let Some(ref mut state) = global_state.as_mut() {
        // Add status message for global search completion
        state
            .app
            .add_status_log("✅ Global search completed".to_string());

        // Create all_reminders with actual list names
        let all_reminders: Vec<(Reminder, String)> = reminders
            .iter()
            .zip(list_names.iter())
            .map(|(reminder, list_name)| (reminder.clone(), list_name.clone()))
            .collect();

        state
            .app
            .set_reminders_with_global_data(reminders, all_reminders);
        Ok(())
    } else {
        Err(RemError::TUIError {
            message: "TUI not initialized".to_string(),
        })
    }
}

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::AppView;

    #[test]
    fn test_toggle_completed_visibility_action() {
        // Test that the ToggleCompletedVisibility action is properly defined
        let action = TuiAction::ToggleCompletedVisibility;

        match action {
            TuiAction::ToggleCompletedVisibility => {
                // Success - the action exists and matches
            }
            _ => {
                panic!("ToggleCompletedVisibility action not found");
            }
        }
    }

    #[test]
    fn test_tui_app_with_reminders() {
        // Create test data
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 3,
        }];

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
            },
        ];

        // Test TUI app creation
        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Test setting reminders
        tui_app.set_reminders(reminders.clone());

        // Test filtering logic
        let filtered_incomplete = tui_app.get_filtered_reminders_for_test();

        // Initially show_completed_todos is false, so should only show incomplete
        assert_eq!(
            filtered_incomplete.len(),
            2,
            "Should show 2 incomplete reminders"
        );

        // Toggle visibility to show completed
        tui_app.set_show_completed_todos(true);
        let filtered_all = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            filtered_all.len(),
            3,
            "Should show all 3 reminders when toggle is on"
        );

        // Toggle back to hide completed
        tui_app.set_show_completed_todos(false);
        let filtered_incomplete_again = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            filtered_incomplete_again.len(),
            2,
            "Should show 2 incomplete reminders again"
        );

        println!("✅ Toggle completed visibility test passed!");
    }

    #[test]
    fn test_default_hide_completed() {
        // Test that completed todos are hidden by default
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#0000FF".to_string(),
            count: 1,
        }];

        let tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // By default, should hide completed todos
        assert!(
            !tui_app.show_completed_todos(),
            "Completed todos should be hidden by default"
        );

        println!("✅ Default hide completed test passed!");
    }

    #[test]
    fn test_toggle_with_completed_reminders() {
        // Test toggling visibility with actual completed reminders
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#00FF00".to_string(),
            count: 4,
        }];

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
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        tui_app.set_reminders(reminders.clone());

        // Initially should show only incomplete (2 reminders)
        let initial_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            initial_filtered.len(),
            2,
            "Should initially show 2 incomplete reminders"
        );
        assert!(
            initial_filtered.iter().all(|r| !r.completed),
            "All shown reminders should be incomplete"
        );

        // Toggle to show completed
        tui_app.set_show_completed_todos(true);
        let all_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            all_filtered.len(),
            4,
            "Should show all 4 reminders when toggle is on"
        );

        // Verify we have both completed and incomplete
        let completed_count = all_filtered.iter().filter(|r| r.completed).count();
        let incomplete_count = all_filtered.iter().filter(|r| !r.completed).count();
        assert_eq!(completed_count, 2, "Should show 2 completed reminders");
        assert_eq!(incomplete_count, 2, "Should show 2 incomplete reminders");

        // Toggle back to hide completed
        tui_app.set_show_completed_todos(false);
        let final_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            final_filtered.len(),
            2,
            "Should show 2 incomplete reminders again"
        );
        assert!(
            final_filtered.iter().all(|r| !r.completed),
            "All shown reminders should be incomplete again"
        );

        println!("✅ Toggle with completed reminders test passed!");
    }

    #[test]
    fn test_search_state_basic_operations() {
        // Test search state initialization and basic operations
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 1,
        }];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Test initial search state
        assert!(
            !tui_app.is_search_active(),
            "Search should be inactive initially"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "",
            "Search query should be empty initially"
        );

        // Test starting global search
        tui_app.start_global_search();
        assert!(
            tui_app.is_search_active(),
            "Search should be active after starting"
        );
        assert!(
            tui_app.is_global_search(),
            "Should be in global search mode"
        );

        // Test search query manipulation
        tui_app.add_search_char('t');
        tui_app.add_search_char('e');
        tui_app.add_search_char('s');
        tui_app.add_search_char('t');
        assert_eq!(
            tui_app.get_search_query(),
            "test",
            "Search query should be 'test'"
        );

        // Test backspace
        tui_app.remove_search_char();
        assert_eq!(
            tui_app.get_search_query(),
            "tes",
            "Search query should be 'tes' after backspace"
        );

        // Test clearing search
        tui_app.clear_search();
        assert!(
            !tui_app.is_search_active(),
            "Search should be inactive after clearing"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "",
            "Search query should be empty after clearing"
        );

        println!("✅ Search state basic operations test passed!");
    }

    #[test]
    fn test_search_filtering() {
        // Test search filtering functionality
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#0000FF".to_string(),
            count: 5,
        }];

        let reminders = vec![
            Reminder {
                id: "rem-1".to_string(),
                title: "Buy groceries".to_string(),
                notes: None,
                completed: false,
                priority: 1,
                due_date: None,
            },
            Reminder {
                id: "rem-2".to_string(),
                title: "Call dentist".to_string(),
                notes: Some("Schedule appointment".to_string()),
                completed: false,
                priority: 2,
                due_date: None,
            },
            Reminder {
                id: "rem-3".to_string(),
                title: "Buy birthday gift".to_string(),
                notes: None,
                completed: true,
                priority: 0,
                due_date: None,
            },
            Reminder {
                id: "rem-4".to_string(),
                title: "Exercise".to_string(),
                notes: Some("30 minutes cardio".to_string()),
                completed: false,
                priority: 3,
                due_date: None,
            },
            Reminder {
                id: "rem-5".to_string(),
                title: "Read book".to_string(),
                notes: None,
                completed: true,
                priority: 1,
                due_date: None,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        tui_app.set_reminders(reminders.clone());

        // Test search for "buy" - should match "Buy groceries" and "Buy birthday gift"
        tui_app.start_list_search();
        tui_app.add_search_char('b');
        tui_app.add_search_char('u');
        tui_app.add_search_char('y');

        let buy_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            buy_results.len(),
            1,
            "Should find 1 reminder with 'buy' (only incomplete by default)"
        );
        assert_eq!(
            buy_results[0].title, "Buy groceries",
            "Should find the incomplete 'Buy groceries' reminder"
        );

        // Enable completed todos and search again
        tui_app.set_show_completed_todos(true);
        let buy_results_with_completed = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            buy_results_with_completed.len(),
            2,
            "Should find 2 reminders with 'buy' when showing completed"
        );

        // Test search for "appointment" in notes
        tui_app.clear_search();
        tui_app.start_list_search();
        tui_app.add_search_char('a');
        tui_app.add_search_char('p');
        tui_app.add_search_char('p');
        tui_app.add_search_char('o');
        tui_app.add_search_char('i');
        tui_app.add_search_char('n');
        tui_app.add_search_char('t');

        let appointment_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            appointment_results.len(),
            1,
            "Should find 1 reminder with 'appointment' in notes"
        );
        assert_eq!(
            appointment_results[0].title, "Call dentist",
            "Should find the 'Call dentist' reminder"
        );

        // Test case insensitive search
        tui_app.clear_search();
        tui_app.start_list_search();
        tui_app.add_search_char('E');
        tui_app.add_search_char('X');
        tui_app.add_search_char('E');
        tui_app.add_search_char('R');

        let exercise_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            exercise_results.len(),
            1,
            "Should find 1 reminder with case insensitive 'EXER' search"
        );
        assert_eq!(
            exercise_results[0].title, "Exercise",
            "Should find the 'Exercise' reminder"
        );

        println!("✅ Search filtering test passed!");
    }

    #[test]
    fn test_search_modes() {
        // Test global vs list-specific search modes
        let lists = vec![
            ReminderList {
                id: "work-list".to_string(),
                name: "Work".to_string(),
                color: "#FF0000".to_string(),
                count: 2,
            },
            ReminderList {
                id: "personal-list".to_string(),
                name: "Personal".to_string(),
                color: "#00FF00".to_string(),
                count: 2,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Test starting global search
        tui_app.start_global_search();
        assert!(tui_app.is_search_active(), "Search should be active");
        assert!(
            tui_app.is_global_search(),
            "Should be in global search mode"
        );

        // Test starting list search
        tui_app.start_list_search();
        assert!(tui_app.is_search_active(), "Search should be active");
        assert!(!tui_app.is_global_search(), "Should be in list search mode");

        // Test exit search (keeps results)
        tui_app.add_search_char('t');
        tui_app.add_search_char('e');
        tui_app.add_search_char('s');
        tui_app.add_search_char('t');
        tui_app.exit_search();
        assert!(
            !tui_app.is_search_active(),
            "Search should be inactive after exit"
        );
        assert!(
            tui_app.has_search_results(),
            "Should still have search results"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "test",
            "Search query should be preserved"
        );

        // Test clear search (removes everything)
        tui_app.clear_search();
        assert!(
            !tui_app.is_search_active(),
            "Search should be inactive after clear"
        );
        assert!(
            !tui_app.has_search_results(),
            "Should not have search results after clear"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "",
            "Search query should be empty after clear"
        );

        println!("✅ Search modes test passed!");
    }

    #[test]
    fn test_global_search_action() {
        // Test GlobalSearch action creation and handling
        let action = TuiAction::GlobalSearch {
            query: "test query".to_string(),
        };

        match action {
            TuiAction::GlobalSearch { query } => {
                assert_eq!(
                    query, "test query",
                    "GlobalSearch action should contain the correct query"
                );
            }
            _ => {
                panic!("GlobalSearch action not found or incorrectly structured");
            }
        }

        println!("✅ GlobalSearch action test passed!");
    }

    #[test]
    fn test_search_with_empty_query() {
        // Test search behavior with empty query
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 2,
        }];

        let reminders = vec![
            Reminder {
                id: "rem-1".to_string(),
                title: "Task 1".to_string(),
                notes: None,
                completed: false,
                priority: 1,
                due_date: None,
            },
            Reminder {
                id: "rem-2".to_string(),
                title: "Task 2".to_string(),
                notes: None,
                completed: false,
                priority: 2,
                due_date: None,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        tui_app.set_reminders(reminders.clone());

        // Start search but don't add any characters
        tui_app.start_list_search();
        assert!(tui_app.is_search_active(), "Search should be active");
        assert!(
            !tui_app.has_search_results(),
            "Should not have results with empty query"
        );

        // Check that all reminders are still shown (no filtering with empty query)
        let all_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            all_results.len(),
            2,
            "Should show all reminders with empty search query"
        );

        // Add character then remove it
        tui_app.add_search_char('t');
        assert!(
            tui_app.has_search_results(),
            "Should have results after adding character"
        );

        tui_app.remove_search_char();
        assert!(
            !tui_app.has_search_results(),
            "Should not have results after removing character"
        );

        println!("✅ Empty query search test passed!");
    }

    #[test]
    fn test_search_filtering_combination() {
        // Test search filtering combined with completed visibility toggle
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#0000FF".to_string(),
            count: 4,
        }];

        let reminders = vec![
            Reminder {
                id: "rem-1".to_string(),
                title: "Task one complete".to_string(),
                notes: None,
                completed: true,
                priority: 1,
                due_date: None,
            },
            Reminder {
                id: "rem-2".to_string(),
                title: "Task one incomplete".to_string(),
                notes: None,
                completed: false,
                priority: 2,
                due_date: None,
            },
            Reminder {
                id: "rem-3".to_string(),
                title: "Task two complete".to_string(),
                notes: None,
                completed: true,
                priority: 0,
                due_date: None,
            },
            Reminder {
                id: "rem-4".to_string(),
                title: "Different task".to_string(),
                notes: None,
                completed: false,
                priority: 3,
                due_date: None,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");
        tui_app.set_reminders(reminders.clone());

        // Search for "one" with completed todos hidden (default)
        tui_app.start_list_search();
        tui_app.add_search_char('o');
        tui_app.add_search_char('n');
        tui_app.add_search_char('e');

        let one_results_incomplete_only = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            one_results_incomplete_only.len(),
            1,
            "Should find 1 incomplete reminder with 'one'"
        );
        assert_eq!(
            one_results_incomplete_only[0].title, "Task one incomplete",
            "Should find the incomplete task"
        );

        // Show completed todos and search again
        tui_app.set_show_completed_todos(true);
        let one_results_all = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            one_results_all.len(),
            2,
            "Should find 2 reminders with 'one' when showing completed"
        );

        // Verify both completed and incomplete are found
        let titles: Vec<&str> = one_results_all.iter().map(|r| r.title.as_str()).collect();
        assert!(
            titles.contains(&"Task one complete"),
            "Should contain completed task"
        );
        assert!(
            titles.contains(&"Task one incomplete"),
            "Should contain incomplete task"
        );

        // Hide completed again and verify filtering
        tui_app.set_show_completed_todos(false);
        let one_results_incomplete_again = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            one_results_incomplete_again.len(),
            1,
            "Should find 1 incomplete reminder again"
        );

        println!("✅ Search filtering combination test passed!");
    }

    #[test]
    fn test_global_search_view_switching() {
        // Test that global search correctly switches views and loads data
        let lists = vec![
            ReminderList {
                id: "work-list".to_string(),
                name: "Work".to_string(),
                color: "#FF0000".to_string(),
                count: 2,
            },
            ReminderList {
                id: "personal-list".to_string(),
                name: "Personal".to_string(),
                color: "#00FF00".to_string(),
                count: 1,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Start from Lists view
        assert!(
            matches!(tui_app.get_current_view(), AppView::Lists),
            "Should start in Lists view"
        );

        // Simulate pressing "/" to start global search
        tui_app.start_global_search();

        // Verify search state
        assert!(tui_app.is_search_active(), "Search should be active");
        assert!(
            tui_app.is_global_search(),
            "Should be in global search mode"
        );

        // Switch to global search view (simulating the key handler)
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });

        // Verify view switched
        assert!(
            tui_app.is_in_global_search_view(),
            "Should be in global search view"
        );

        println!("✅ Global search view switching test passed!");
    }

    #[test]
    fn test_global_search_data_handling() {
        // Test global search data loading and filtering
        let lists = vec![ReminderList {
            id: "list1".to_string(),
            name: "List 1".to_string(),
            color: "#FF0000".to_string(),
            count: 2,
        }];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Set up global reminders with list names
        let global_reminders = vec![
            (
                Reminder {
                    id: "rem-1".to_string(),
                    title: "Work task".to_string(),
                    notes: None,
                    completed: false,
                    priority: 1,
                    due_date: None,
                },
                "Work".to_string(),
            ),
            (
                Reminder {
                    id: "rem-2".to_string(),
                    title: "Personal task".to_string(),
                    notes: None,
                    completed: false,
                    priority: 2,
                    due_date: None,
                },
                "Personal".to_string(),
            ),
            (
                Reminder {
                    id: "rem-3".to_string(),
                    title: "Shopping list".to_string(),
                    notes: Some("Buy groceries".to_string()),
                    completed: true,
                    priority: 0,
                    due_date: None,
                },
                "Personal".to_string(),
            ),
        ];

        tui_app.set_all_reminders(global_reminders);

        // Start global search
        tui_app.start_global_search();
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });

        // Test that global reminders are used for filtering
        let all_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            all_filtered.len(),
            2,
            "Should show 2 incomplete reminders by default"
        );

        // Test global search filtering
        tui_app.add_search_char('w');
        tui_app.add_search_char('o');
        tui_app.add_search_char('r');
        tui_app.add_search_char('k');

        let work_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(work_filtered.len(), 1, "Should find 1 reminder with 'work'");
        assert_eq!(
            work_filtered[0].title, "Work task",
            "Should find the work task"
        );

        // Test that completed todos can be shown in global search
        tui_app.set_show_completed_todos(true);
        let work_with_completed = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            work_with_completed.len(),
            1,
            "Should still find 1 reminder with 'work' including completed"
        );

        // Clear search and search for something that matches completed items
        tui_app.clear_search();
        tui_app.start_global_search();
        tui_app.add_search_char('s');
        tui_app.add_search_char('h');
        tui_app.add_search_char('o');
        tui_app.add_search_char('p');

        let shop_filtered = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            shop_filtered.len(),
            1,
            "Should find completed shopping task"
        );
        assert_eq!(
            shop_filtered[0].title, "Shopping list",
            "Should find the shopping task"
        );

        println!("✅ Global search data handling test passed!");
    }

    #[test]
    fn test_global_search_list_name_lookup() {
        // Test the helper method for getting list names
        let lists = vec![ReminderList {
            id: "list1".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 1,
        }];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Set up global reminders with list names
        let global_reminders = vec![
            (
                Reminder {
                    id: "rem-1".to_string(),
                    title: "Task 1".to_string(),
                    notes: None,
                    completed: false,
                    priority: 1,
                    due_date: None,
                },
                "Work List".to_string(),
            ),
            (
                Reminder {
                    id: "rem-2".to_string(),
                    title: "Task 2".to_string(),
                    notes: None,
                    completed: false,
                    priority: 2,
                    due_date: None,
                },
                "Personal List".to_string(),
            ),
        ];

        tui_app.set_all_reminders(global_reminders);

        // Test list name lookup
        let list_name_1 = tui_app.get_list_name_for_reminder("rem-1");
        assert_eq!(
            list_name_1,
            Some("Work List"),
            "Should find list name for rem-1"
        );

        let list_name_2 = tui_app.get_list_name_for_reminder("rem-2");
        assert_eq!(
            list_name_2,
            Some("Personal List"),
            "Should find list name for rem-2"
        );

        let list_name_none = tui_app.get_list_name_for_reminder("nonexistent");
        assert_eq!(
            list_name_none, None,
            "Should return None for nonexistent reminder"
        );

        println!("✅ Global search list name lookup test passed!");
    }

    #[test]
    fn test_global_search_escape_behavior() {
        // Test escape key behavior in global search
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 1,
        }];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Start global search and add a query
        tui_app.start_global_search();
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });
        tui_app.add_search_char('t');
        tui_app.add_search_char('e');
        tui_app.add_search_char('s');
        tui_app.add_search_char('t');

        assert!(tui_app.is_search_active(), "Search should be active");
        assert!(tui_app.has_search_results(), "Should have search results");
        assert_eq!(
            tui_app.get_search_query(),
            "test",
            "Search query should be 'test'"
        );

        // Test exit search (first escape)
        tui_app.exit_search();
        assert!(
            !tui_app.is_search_active(),
            "Search should not be active after exit"
        );
        assert!(
            tui_app.has_search_results(),
            "Should still have search results after exit"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "test",
            "Search query should be preserved after exit"
        );

        // Test clear search (second escape equivalent)
        tui_app.clear_search();
        assert!(
            !tui_app.is_search_active(),
            "Search should not be active after clear"
        );
        assert!(
            !tui_app.has_search_results(),
            "Should not have search results after clear"
        );
        assert_eq!(
            tui_app.get_search_query(),
            "",
            "Search query should be empty after clear"
        );
        assert!(
            !tui_app.is_global_search(),
            "Should not be in global search mode after clear"
        );

        println!("✅ Global search escape behavior test passed!");
    }

    #[test]
    fn test_global_search_integration() {
        // Integration test for complete global search flow
        let lists = vec![
            ReminderList {
                id: "work".to_string(),
                name: "Work".to_string(),
                color: "#FF0000".to_string(),
                count: 2,
            },
            ReminderList {
                id: "personal".to_string(),
                name: "Personal".to_string(),
                color: "#00FF00".to_string(),
                count: 1,
            },
        ];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Start in Lists view
        assert!(
            matches!(tui_app.get_current_view(), AppView::Lists),
            "Should start in Lists view"
        );

        // Simulate "/" key press from Lists view
        tui_app.start_global_search();
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });

        // Verify we're in global search mode
        assert!(tui_app.is_search_active(), "Should be in search mode");
        assert!(
            tui_app.is_global_search(),
            "Should be in global search mode"
        );
        assert!(
            tui_app.is_in_global_search_view(),
            "Should be in global search view"
        );

        // Set up mock global data
        let global_reminders = vec![
            (
                Reminder {
                    id: "work-1".to_string(),
                    title: "Prepare presentation".to_string(),
                    notes: Some("For Monday meeting".to_string()),
                    completed: false,
                    priority: 2,
                    due_date: None,
                },
                "Work".to_string(),
            ),
            (
                Reminder {
                    id: "personal-1".to_string(),
                    title: "Buy groceries".to_string(),
                    notes: None,
                    completed: false,
                    priority: 1,
                    due_date: None,
                },
                "Personal".to_string(),
            ),
            (
                Reminder {
                    id: "work-2".to_string(),
                    title: "Review code".to_string(),
                    notes: None,
                    completed: true,
                    priority: 1,
                    due_date: None,
                },
                "Work".to_string(),
            ),
        ];

        tui_app.set_all_reminders(global_reminders);

        // Test initial state - should show all incomplete reminders
        let initial_reminders = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            initial_reminders.len(),
            2,
            "Should show 2 incomplete reminders initially"
        );

        // Test searching for "buy"
        tui_app.add_search_char('b');
        tui_app.add_search_char('u');
        tui_app.add_search_char('y');

        let buy_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(buy_results.len(), 1, "Should find 1 reminder with 'buy'");
        assert_eq!(
            buy_results[0].title, "Buy groceries",
            "Should find the groceries reminder"
        );

        // Test searching in notes
        tui_app.clear_search();
        tui_app.start_global_search();
        tui_app.add_search_char('m');
        tui_app.add_search_char('e');
        tui_app.add_search_char('e');
        tui_app.add_search_char('t');

        let meeting_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(
            meeting_results.len(),
            1,
            "Should find 1 reminder with 'meet' in notes"
        );
        assert_eq!(
            meeting_results[0].title, "Prepare presentation",
            "Should find the presentation reminder"
        );

        // Test showing completed todos
        tui_app.clear_search();
        tui_app.start_global_search();
        tui_app.set_show_completed_todos(true);
        tui_app.add_search_char('r');
        tui_app.add_search_char('e');
        tui_app.add_search_char('v');

        let review_results = tui_app.get_filtered_reminders_for_test();
        assert_eq!(review_results.len(), 1, "Should find completed review task");
        assert_eq!(
            review_results[0].title, "Review code",
            "Should find the review reminder"
        );
        assert!(
            review_results[0].completed,
            "Found reminder should be completed"
        );

        println!("✅ Global search integration test passed!");
    }

    #[test]
    fn test_global_search_navigation_back() {
        // Test that escape and 'q' keys work to go back from global search
        let lists = vec![ReminderList {
            id: "test-list".to_string(),
            name: "Test List".to_string(),
            color: "#FF0000".to_string(),
            count: 1,
        }];

        let mut tui_app = TUIApp::new(lists).expect("Failed to create TUI app");

        // Start in Lists view
        assert!(
            matches!(tui_app.get_current_view(), AppView::Lists),
            "Should start in Lists view"
        );

        // Start global search
        tui_app.start_global_search();
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });

        // Verify we're in global search
        assert!(tui_app.is_search_active(), "Should be in search mode");
        assert!(
            tui_app.is_in_global_search_view(),
            "Should be in global search view"
        );

        // Test that clearing search goes back to Lists
        tui_app.clear_search();
        assert!(!tui_app.is_search_active(), "Search should be cleared");
        assert!(
            !tui_app.is_global_search(),
            "Should not be in global search mode"
        );

        // Manual navigation back test (simulating what happens in key handler)
        tui_app.start_global_search();
        tui_app.set_current_view(AppView::Reminders {
            list_id: "global".to_string(),
        });

        // Simulate escape key behavior
        tui_app.set_current_view(AppView::Lists);
        tui_app.clear_search();

        assert!(
            matches!(tui_app.get_current_view(), AppView::Lists),
            "Should be back in Lists view"
        );
        assert!(!tui_app.is_search_active(), "Search should be cleared");

        println!("✅ Global search navigation back test passed!");
    }
}
