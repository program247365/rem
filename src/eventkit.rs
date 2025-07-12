use color_eyre::Result;
use objc::runtime::{Class, Object, BOOL, YES};
use objc::{msg_send, sel, sel_impl};
use std::ptr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// Macro for conditional debug logging based on DEBUG environment variable
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var("DEBUG").unwrap_or_default() == "true" {
            eprintln!($($arg)*);
        }
    };
}

// Force link EventKit framework
#[link(name = "EventKit", kind = "framework")]
#[link(name = "Foundation", kind = "framework")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderList {
    pub id: String,
    pub title: String,
    pub color: String,
    pub reminder_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub completed: bool,
    pub priority: u8,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    NotDetermined,
    Denied,
    Authorized,
    Restricted,
}

impl From<i64> for PermissionStatus {
    fn from(value: i64) -> Self {
        match value {
            0 => PermissionStatus::NotDetermined,
            1 => PermissionStatus::Restricted,
            2 => PermissionStatus::Denied,
            3 => PermissionStatus::Authorized,
            _ => PermissionStatus::NotDetermined,
        }
    }
}

pub struct EventKitManager {
    event_store: *mut Object,
}

impl EventKitManager {
    pub fn new() -> Result<Self> {
        unsafe {
            // Check if EventKit framework is available
            let event_store_class = Class::get("EKEventStore")
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get EKEventStore class - EventKit framework may not be linked or available"))?;
            
            let event_store: *mut Object = msg_send![event_store_class, new];
            
            if event_store.is_null() {
                return Err(color_eyre::eyre::eyre!("Failed to create EKEventStore instance"));
            }
            
            // Test basic functionality
            let calendars: *mut Object = msg_send![event_store, calendarsForEntityType: 1i64];
            if calendars.is_null() {
                return Err(color_eyre::eyre::eyre!("Failed to access calendars - EventKit may not be properly initialized"));
            }
            
            Ok(EventKitManager { event_store })
        }
    }

    pub fn check_permission_status(&self) -> PermissionStatus {
        unsafe {
            let event_store_class = match Class::get("EKEventStore") {
                Some(class) => class,
                None => return PermissionStatus::NotDetermined,
            };
            let entity_type_reminder: i64 = 1; // EKEntityTypeReminder
            let status: i64 = msg_send![event_store_class, authorizationStatusForEntityType: entity_type_reminder];
            PermissionStatus::from(status)
        }
    }

    pub async fn request_permission(&self) -> Result<bool> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        
        unsafe {
            let entity_type_reminder: i64 = 1; // EKEntityTypeReminder
            
            // Create a completion block using ConcreteBlock
            let completion_block = block::ConcreteBlock::new(move |granted: BOOL, _error: *mut Object| {
                if let Ok(mut tx_guard) = tx.lock() {
                    if let Some(tx) = tx_guard.take() {
                        let _ = tx.send(granted == YES);
                    }
                }
            });
            
            let completion_block = completion_block.copy();
            
            let _: () = msg_send![self.event_store, 
                requestAccessToEntityType: entity_type_reminder 
                completion: completion_block
            ];
        }
        
        // Wait for the callback with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(granted)) => Ok(granted),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false), // Timeout
        }
    }

    pub fn get_reminder_lists(&self) -> Result<Vec<ReminderList>> {
        unsafe {
            debug_log!("Debug: Getting reminder lists...");
            let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];
            let count: usize = msg_send![calendars, count];
            debug_log!("Debug: Found {} calendars", count);
            
            let mut lists = Vec::new();
            
            for i in 0..count {
                let calendar: *mut Object = msg_send![calendars, objectAtIndex: i];
                let title_nsstring: *mut Object = msg_send![calendar, title];
                let calendar_identifier: *mut Object = msg_send![calendar, calendarIdentifier];
                
                // Convert NSString to Rust String
                let title_ptr: *const i8 = msg_send![title_nsstring, UTF8String];
                let title = std::ffi::CStr::from_ptr(title_ptr)
                    .to_string_lossy()
                    .into_owned();
                
                let id_ptr: *const i8 = msg_send![calendar_identifier, UTF8String];
                let id = std::ffi::CStr::from_ptr(id_ptr)
                    .to_string_lossy()
                    .into_owned();
                
                debug_log!("Debug: Processing calendar '{}' with ID '{}'", title, id);
                
                // Get color (simplified - generating a color based on index)
                let color_string = format!(
                    "#{:02x}{:02x}{:02x}",
                    (i * 60 + 100) % 255,
                    (i * 120 + 50) % 255,
                    (i * 180 + 150) % 255
                );
                
                // Get reminder count for this list
                let reminder_count = self.get_reminder_count_for_list(&id)?;
                debug_log!("Debug: Calendar '{}' has {} reminders", title, reminder_count);
                
                lists.push(ReminderList {
                    id,
                    title,
                    color: color_string,
                    reminder_count,
                });
            }
            
            debug_log!("Debug: Returning {} reminder lists", lists.len());
            Ok(lists)
        }
    }

    pub fn get_reminders_for_list(&self, list_id: &str) -> Result<Vec<Reminder>> {
        debug_log!("Debug: Getting reminders for list ID using AppleScript: {}", list_id);
        
        // First get the list name from EventKit
        let list_name = unsafe {
            let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];
            let count: usize = msg_send![calendars, count];
            
            let mut found_title = "Unknown".to_string();
            for i in 0..count {
                let calendar: *mut Object = msg_send![calendars, objectAtIndex: i];
                let calendar_identifier: *mut Object = msg_send![calendar, calendarIdentifier];
                let id_ptr: *const i8 = msg_send![calendar_identifier, UTF8String];
                let id = std::ffi::CStr::from_ptr(id_ptr).to_string_lossy();
                
                if id == list_id {
                    let title_nsstring: *mut Object = msg_send![calendar, title];
                    let title_ptr: *const i8 = msg_send![title_nsstring, UTF8String];
                    let title = std::ffi::CStr::from_ptr(title_ptr).to_string_lossy().into_owned();
                    debug_log!("Debug: Found calendar name for reminders: {}", title);
                    found_title = title;
                    break;
                }
            }
            found_title
        };
        
        // Use AppleScript to get reminders data
        let script = format!(
            r#"tell application "Reminders"
                try
                    set reminderList to list "{}"
                    set reminderItems to reminders in reminderList
                    set resultList to {{}}
                    
                    repeat with aReminder in reminderItems
                        set reminderName to name of aReminder
                        set reminderCompleted to completed of aReminder
                        set reminderBody to body of aReminder
                        
                        -- Format: "NAME|COMPLETED|BODY"
                        if reminderBody is missing value then
                            set reminderData to reminderName & "|" & (reminderCompleted as string) & "|"
                        else
                            set reminderData to reminderName & "|" & (reminderCompleted as string) & "|" & reminderBody
                        end if
                        
                        set end of resultList to reminderData
                    end repeat
                    
                    set AppleScript's text item delimiters to "~REMINDER~"
                    set resultString to resultList as string
                    set AppleScript's text item delimiters to ""
                    
                    return resultString
                on error errorMessage
                    return "ERROR:" & errorMessage
                end try
            end tell"#,
            list_name.replace("\"", "\\\"")
        );
        
        debug_log!("Debug: Running AppleScript for reminders");
        
        match std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
        {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                debug_log!("Debug: AppleScript reminders result length: {}", result.len());
                
                if result.starts_with("ERROR:") {
                    debug_log!("Debug: AppleScript error: {}", result);
                    return Ok(Vec::new());
                }
                
                let mut reminders = Vec::new();
                let reminder_data_list: Vec<&str> = result.split("~REMINDER~").collect();
                
                for (index, reminder_data) in reminder_data_list.iter().enumerate() {
                    if reminder_data.is_empty() {
                        continue;
                    }
                    
                    let parts: Vec<&str> = reminder_data.split('|').collect();
                    if parts.len() >= 2 {
                        let title = parts[0].to_string();
                        let completed = parts[1] == "true";
                        let notes = if parts.len() > 2 && !parts[2].is_empty() {
                            Some(parts[2].to_string())
                        } else {
                            None
                        };
                        
                        reminders.push(Reminder {
                            id: format!("reminder-{}-{}", list_name.to_lowercase(), index),
                            title,
                            notes,
                            completed,
                            priority: 0, // AppleScript doesn't easily expose priority
                            due_date: None, // We could add due date parsing later
                        });
                        
                        debug_log!("Debug: Added AppleScript reminder: {}", reminders.last().unwrap().title);
                    }
                }
                
                debug_log!("Debug: Returning {} AppleScript reminders", reminders.len());
                Ok(reminders)
            }
            Err(e) => {
                debug_log!("Debug: AppleScript execution failed: {}", e);
                Ok(Vec::new())
            }
        }
    }

    fn get_reminder_count_for_list(&self, list_id: &str) -> Result<usize> {
        debug_log!("Debug: Getting reminder count for list ID using AppleScript: {}", list_id);
        
        // First get the list name from EventKit
        let list_name = unsafe {
            let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];
            let count: usize = msg_send![calendars, count];
            
            let mut found_title = "Unknown".to_string();
            for i in 0..count {
                let calendar: *mut Object = msg_send![calendars, objectAtIndex: i];
                let calendar_identifier: *mut Object = msg_send![calendar, calendarIdentifier];
                let id_ptr: *const i8 = msg_send![calendar_identifier, UTF8String];
                let id = std::ffi::CStr::from_ptr(id_ptr).to_string_lossy();
                
                if id == list_id {
                    let title_nsstring: *mut Object = msg_send![calendar, title];
                    let title_ptr: *const i8 = msg_send![title_nsstring, UTF8String];
                    let title = std::ffi::CStr::from_ptr(title_ptr).to_string_lossy().into_owned();
                    debug_log!("Debug: Found calendar name: {}", title);
                    found_title = title;
                    break;
                }
            }
            found_title
        };
        
        // Use AppleScript to get reminder count
        let script = format!(
            r#"tell application "Reminders"
                try
                    set listCount to count of reminders in list "{}"
                    return listCount as string
                on error
                    return "0"
                end try
            end tell"#,
            list_name.replace("\"", "\\\"")
        );
        
        debug_log!("Debug: Running AppleScript: {}", script);
        
        match std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
        {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                debug_log!("Debug: AppleScript result: '{}'", result);
                
                if let Ok(count) = result.parse::<usize>() {
                    debug_log!("Debug: Successfully parsed count: {}", count);
                    Ok(count)
                } else {
                    debug_log!("Debug: Failed to parse count, returning 0");
                    Ok(0)
                }
            }
            Err(e) => {
                debug_log!("Debug: AppleScript execution failed: {}", e);
                Ok(0)
            }
        }
    }
}

impl Drop for EventKitManager {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.event_store, release];
        }
    }
}

// Make it thread-safe
unsafe impl Send for EventKitManager {}
unsafe impl Sync for EventKitManager {}