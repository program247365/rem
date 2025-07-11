use color_eyre::Result;
use objc::runtime::{Class, Object, BOOL, YES};
use objc::{msg_send, sel, sel_impl};
use std::ptr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

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
            let event_store_class = Class::get("EKEventStore")
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get EKEventStore class"))?;
            let event_store: *mut Object = msg_send![event_store_class, new];
            
            if event_store.is_null() {
                return Err(color_eyre::eyre::eyre!("Failed to create EKEventStore instance"));
            }
            
            Ok(EventKitManager { event_store })
        }
    }

    pub fn check_permission_status(&self) -> PermissionStatus {
        unsafe {
            let event_store_class = Class::get("EKEventStore").unwrap();
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
            let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];
            let count: usize = msg_send![calendars, count];
            
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
                
                // Get color (simplified - generating a color based on index)
                let color_string = format!(
                    "#{:02x}{:02x}{:02x}",
                    (i * 60 + 100) % 255,
                    (i * 120 + 50) % 255,
                    (i * 180 + 150) % 255
                );
                
                // Get reminder count for this list
                let reminder_count = self.get_reminder_count_for_list(&id)?;
                
                lists.push(ReminderList {
                    id,
                    title,
                    color: color_string,
                    reminder_count,
                });
            }
            
            Ok(lists)
        }
    }

    pub fn get_reminders_for_list(&self, list_id: &str) -> Result<Vec<Reminder>> {
        unsafe {
            let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];
            let count: usize = msg_send![calendars, count];
            
            // Find the calendar with matching ID
            let mut target_calendar: *mut Object = ptr::null_mut();
            for i in 0..count {
                let calendar: *mut Object = msg_send![calendars, objectAtIndex: i];
                let calendar_identifier: *mut Object = msg_send![calendar, calendarIdentifier];
                let id_ptr: *const i8 = msg_send![calendar_identifier, UTF8String];
                let id = std::ffi::CStr::from_ptr(id_ptr).to_string_lossy();
                
                if id == list_id {
                    target_calendar = calendar;
                    break;
                }
            }
            
            if target_calendar.is_null() {
                return Ok(Vec::new());
            }
            
            // Create NSArray with the calendar - simplified approach
            let calendar_array: *mut Object = unsafe {
                let array_class = Class::get("NSArray").unwrap();
                let array: *mut Object = msg_send![array_class, arrayWithObject: target_calendar];
                array
            };
            
            // Create predicate for reminders in this calendar
            let predicate: *mut Object = msg_send![self.event_store, 
                predicateForRemindersInCalendars: calendar_array
            ];
            
            // Use fetchRemindersMatchingPredicate to get reminders
            // This is async in EventKit, so we'll use a simplified synchronous approach
            let (tx, rx) = std::sync::mpsc::channel();
            let tx = Arc::new(Mutex::new(Some(tx)));
            
            let completion_block = block::ConcreteBlock::new(move |reminders: *mut Object| {
                if let Ok(mut tx_guard) = tx.lock() {
                    if let Some(tx) = tx_guard.take() {
                        let _ = tx.send(reminders);
                    }
                }
            });
            
            let completion_block = completion_block.copy();
            
            let _: () = msg_send![self.event_store, 
                fetchRemindersMatchingPredicate: predicate 
                completion: completion_block
            ];
            
            // Wait for the result (with timeout)
            match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(reminders_array) => {
                    let count: usize = msg_send![reminders_array, count];
                    let mut reminders = Vec::new();
                    
                    for i in 0..count {
                        let reminder: *mut Object = msg_send![reminders_array, objectAtIndex: i];
                        
                        // Extract reminder data
                        let title_nsstring: *mut Object = msg_send![reminder, title];
                        let title_ptr: *const i8 = msg_send![title_nsstring, UTF8String];
                        let title = std::ffi::CStr::from_ptr(title_ptr)
                            .to_string_lossy()
                            .into_owned();
                        
                        let notes_nsstring: *mut Object = msg_send![reminder, notes];
                        let notes = if !notes_nsstring.is_null() {
                            let notes_ptr: *const i8 = msg_send![notes_nsstring, UTF8String];
                            Some(std::ffi::CStr::from_ptr(notes_ptr)
                                .to_string_lossy()
                                .into_owned())
                        } else {
                            None
                        };
                        
                        let completed: BOOL = msg_send![reminder, isCompleted];
                        let priority: i64 = msg_send![reminder, priority];
                        
                        let reminder_id_nsstring: *mut Object = msg_send![reminder, calendarItemIdentifier];
                        let id_ptr: *const i8 = msg_send![reminder_id_nsstring, UTF8String];
                        let id = std::ffi::CStr::from_ptr(id_ptr)
                            .to_string_lossy()
                            .into_owned();
                        
                        // Get due date
                        let due_date_obj: *mut Object = msg_send![reminder, dueDateComponents];
                        let due_date = if !due_date_obj.is_null() {
                            Some("Due date set".to_string()) // Simplified
                        } else {
                            None
                        };
                        
                        reminders.push(Reminder {
                            id,
                            title,
                            notes,
                            completed: completed == YES,
                            priority: priority.min(3) as u8,
                            due_date,
                        });
                    }
                    
                    Ok(reminders)
                }
                Err(_) => Ok(Vec::new()), // Timeout
            }
        }
    }

    fn get_reminder_count_for_list(&self, _list_id: &str) -> Result<usize> {
        // For now, return 0 - in a real implementation you'd count the reminders
        Ok(0)
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