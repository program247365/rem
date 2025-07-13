# Development Guide

This guide explains how to add new functionality to Rem through all layers of the Swift wrapper + Rust core architecture.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Adding New TUI Actions](#adding-new-tui-actions)
3. [Adding Swift Backend Services](#adding-swift-backend-services)
4. [Step-by-Step Example: Delete Functionality](#step-by-step-example-delete-functionality)
5. [Testing New Features](#testing-new-features)
6. [Common Patterns](#common-patterns)
7. [Troubleshooting](#troubleshooting)

## Architecture Overview

Rem uses a layered architecture where each layer has specific responsibilities:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Swift Layer (System)                       │
│  • Native macOS integration (EventKit)                         │
│  • Permission management                                        │
│  • Data transformation                                          │
│  • Action processing                                            │
└─────────────────────────────────────────────────────────────────┘
                                ↕ UniFFI Bridge
┌─────────────────────────────────────────────────────────────────┐
│                    Rust Core (UI & Logic)                      │
│  • Terminal user interface (Ratatui)                           │
│  • User input handling                                         │
│  • Action generation                                           │
│  • State management                                            │
└─────────────────────────────────────────────────────────────────┘
```

## Adding New TUI Actions

### Step 1: Define the Action in Rust

**File**: `rust-core/src/lib.rs`

```rust
#[derive(uniffi::Enum, Clone, Debug)]
pub enum TuiAction {
    // Existing actions
    Quit,
    SelectList { list_id: String },
    ToggleReminder { reminder_id: String },
    DeleteReminder { reminder_id: String },
    
    // Add your new action here
    YourNewAction { parameter: String },
    
    Back,
    Refresh,
}
```

### Step 2: Add Action to UniFFI Interface

**File**: `rust-core/src/rem_core.udl`

```udl
[Enum]
interface TuiAction {
    Quit();
    SelectList(string list_id);
    ToggleReminder(string reminder_id);
    DeleteReminder(string reminder_id);
    
    // Add your new action here
    YourNewAction(string parameter);
    
    Back();
    Refresh();
};
```

### Step 3: Implement Key Handling in TUI

**File**: `rust-core/src/tui/app.rs`

Add key handling logic in the appropriate key event handler:

```rust
fn handle_reminders_key_event(&mut self, key: crossterm::event::KeyEvent, _list_id: String) {
    match key.code {
        // Existing key handlers...
        
        KeyCode::Char('y') => {
            // Your new key binding
            if let Some(reminder) = self.current_reminders.get(self.selected_index) {
                self.actions.push(TuiAction::YourNewAction { 
                    parameter: reminder.id.clone() 
                });
            }
        }
        
        // Other handlers...
        _ => {}
    }
}
```

### Step 4: Update UI Instructions

Add your new command to the instructions displayed in the TUI:

```rust
// In the render_reminders function
let instructions = Paragraph::new(vec![Line::from(vec![
    // Existing instructions...
    Span::styled("y", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
    Span::styled(" your action  ", Style::default().fg(Color::Gray)),
    // More instructions...
])])
```

### Step 5: Handle Action in Swift

**File**: `RemTUI/Sources/main.swift`

Add action handling in the `processActions` function:

```swift
private static func processActions(
    _ actions: [TuiAction],
    remindersService: RemindersService,
    lists: [ReminderList]
) async -> Bool {
    for action in actions {
        switch action {
        // Existing cases...
        
        case .yourNewAction(let parameter):
            do {
                try await remindersService.yourNewMethod(parameter)
                print("✅ Your action completed")
            } catch {
                print("❌ Error performing your action: \(error)")
            }
            
        // Other cases...
        }
    }
    return true
}
```

### Step 6: Rebuild UniFFI Bindings

```bash
make build-uniffi
```

This regenerates the Swift bindings with your new action.

## Adding Swift Backend Services

### Step 1: Add Method to RemindersService

**File**: `RemTUIKit/Sources/RemTUIKit/RemindersService.swift`

```swift
public func yourNewMethod(_ parameter: String) async throws {
    // Find the item using EventKit
    let calendars = eventStore.calendars(for: .reminder)
    
    for calendar in calendars {
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        let reminders = try await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders ?? [])
            }
        }
        
        if let reminder = reminders.first(where: { $0.calendarItemIdentifier == parameter }) {
            // Perform your action using EventKit
            // Example: reminder.priority = EKReminderPriority.high.rawValue
            
            try eventStore.save(reminder, commit: true)
            return
        }
    }
    
    throw RemError.DataAccessError(message: "Item not found")
}
```

### Step 2: Handle Errors Appropriately

Make sure to throw appropriate `RemError` types:

```swift
// Permission issues
throw RemError.PermissionDenied

// Data access problems  
throw RemError.DataAccessError(message: "Specific error description")

// Other issues
throw RemError.DataAccessError(message: "General error message")
```

## Step-by-Step Example: Delete Functionality

Let's walk through how the delete functionality was implemented as a complete example.

### 1. Rust Action Definition

```rust
// In rust-core/src/lib.rs
#[derive(uniffi::Enum, Clone, Debug)]
pub enum TuiAction {
    // ... other actions
    DeleteReminder { reminder_id: String },  // ← Added this
    // ... more actions
}
```

### 2. UniFFI Interface Update

```udl
// In rust-core/src/rem_core.udl
[Enum]
interface TuiAction {
    // ... other actions
    DeleteReminder(string reminder_id);  // ← Added this
    // ... more actions
};
```

### 3. TUI State for Key Sequences

```rust
// In rust-core/src/tui/app.rs
pub struct TUIApp {
    // ... existing fields
    last_key: Option<KeyCode>,  // ← Added for 'dd' detection
}

impl TUIApp {
    pub fn new(lists: Vec<ReminderList>) -> Result<Self, RemError> {
        Ok(Self {
            // ... existing fields
            last_key: None,  // ← Initialize
        })
    }
}
```

### 4. Key Event Handling

```rust
// In rust-core/src/tui/app.rs
fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
    match &self.current_view {
        AppView::Lists => self.handle_lists_key_event(key),
        AppView::Reminders { list_id } => self.handle_reminders_key_event(key, list_id.clone()),
    }
    
    // Update last key for sequence tracking
    self.last_key = Some(key.code);  // ← Added this
}

fn handle_reminders_key_event(&mut self, key: crossterm::event::KeyEvent, _list_id: String) {
    match key.code {
        // ... existing handlers
        
        KeyCode::Char('d') => {
            // Check if this is the second 'd' for 'dd' delete command
            if let Some(KeyCode::Char('d')) = self.last_key {
                if let Some(reminder) = self.current_reminders.get(self.selected_index) {
                    self.actions.push(TuiAction::DeleteReminder { 
                        reminder_id: reminder.id.clone() 
                    });
                }
            }
        }
        
        // ... other handlers
    }
}
```

### 5. UI Instructions Update

```rust
// In rust-core/src/tui/app.rs - render_reminders function
let instructions = Paragraph::new(vec![Line::from(vec![
    // ... existing instructions
    Span::styled("dd", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).bg(Color::DarkGray)),
    Span::styled(" delete  ", Style::default().fg(Color::Gray)),
    // ... more instructions
])])
```

### 6. Swift Service Implementation

```swift
// In RemTUIKit/Sources/RemTUIKit/RemindersService.swift
public func deleteReminder(_ reminderId: String) async throws {
    let calendars = eventStore.calendars(for: .reminder)
    
    for calendar in calendars {
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        let reminders = try await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders ?? [])
            }
        }
        
        if let reminder = reminders.first(where: { $0.calendarItemIdentifier == reminderId }) {
            try eventStore.remove(reminder, commit: true)
            return
        }
    }
    
    throw RemError.DataAccessError(message: "Reminder not found")
}
```

### 7. Swift Action Processing

```swift
// In RemTUI/Sources/main.swift
private static func processActions(
    _ actions: [TuiAction],
    remindersService: RemindersService,
    lists: [ReminderList]
) async -> Bool {
    for action in actions {
        switch action {
        // ... existing cases
        
        case .deleteReminder(let reminderId):
            do {
                try await remindersService.deleteReminder(reminderId)
                print("🗑️ Reminder deleted")
            } catch {
                print("❌ Error deleting reminder: \(error)")
            }
            
        // ... other cases
        }
    }
    return true
}
```

### 8. Build and Test

```bash
# Rebuild UniFFI bindings
make build-uniffi

# Test the functionality
make run-direct
```

## Testing New Features

### 1. Unit Testing

**Rust Tests**:
```bash
make test-rust
```

**Swift Tests**:
```bash
make test-swift
```

### 2. Integration Testing

```bash
make test-integration
```

### 3. Manual Testing

1. Build the application: `make build`
2. Run with direct mode: `make run-direct`
3. Test your new functionality in the TUI
4. Verify actions are processed correctly

### 4. Debugging

**Rust debugging**:
```bash
make debug-rust
```

**Swift debugging**:
```bash
make debug-swift
```

**Full debug mode**:
```bash
make debug
```

## Common Patterns

### Pattern 1: Simple Action (No Parameters)

**Rust Action**:
```rust
pub enum TuiAction {
    SimpleAction,
}
```

**UDL**:
```udl
interface TuiAction {
    SimpleAction();
}
```

**Key Handling**:
```rust
KeyCode::Char('s') => {
    self.actions.push(TuiAction::SimpleAction);
}
```

### Pattern 2: Action with Data

**Rust Action**:
```rust
pub enum TuiAction {
    ActionWithData { id: String, value: u32 },
}
```

**UDL**:
```udl
interface TuiAction {
    ActionWithData(string id, u32 value);
}
```

### Pattern 3: Key Sequences

For multi-key sequences like 'dd', 'gg', etc.:

```rust
pub struct TUIApp {
    last_key: Option<KeyCode>,
    // Can extend to support longer sequences:
    key_sequence: Vec<KeyCode>,
}

fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Char('d') => {
            if let Some(KeyCode::Char('d')) = self.last_key {
                // Execute 'dd' action
            }
        }
        KeyCode::Char('g') => {
            if let Some(KeyCode::Char('g')) = self.last_key {
                // Execute 'gg' action (go to top)
            }
        }
        _ => {
            // Reset sequence for non-sequence keys
        }
    }
    
    self.last_key = Some(key.code);
}
```

### Pattern 4: Modal Operations

For operations that require confirmation:

```rust
#[derive(Clone, Debug)]
enum AppView {
    Lists,
    Reminders { list_id: String },
    ConfirmDelete { reminder_id: String },  // ← Add confirmation modal
}

fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
    match &self.current_view {
        AppView::ConfirmDelete { reminder_id } => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.actions.push(TuiAction::DeleteReminder { 
                        reminder_id: reminder_id.clone() 
                    });
                    self.current_view = AppView::Reminders { /* ... */ };
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.current_view = AppView::Reminders { /* ... */ };
                }
                _ => {}
            }
        }
        // ... other views
    }
}
```

## Troubleshooting

### UniFFI Checksum Mismatches

When you add new actions or modify enums, you may get checksum mismatches:

```bash
# Clean rebuild
make clean-all
make build

# If checksums are still wrong
make fix-checksums
```

### Build Errors

**Missing enum cases**: Make sure you've added the new action to both:
- `rust-core/src/lib.rs` (Rust enum)
- `rust-core/src/rem_core.udl` (UniFFI interface)

**Swift compilation errors**: Rebuild UniFFI bindings:
```bash
make build-uniffi
```

### Runtime Issues

**Action not triggered**: Check that:
1. Key handling is correctly implemented
2. Action is being pushed to `self.actions`
3. Swift side has a matching case statement

**EventKit errors**: Make sure:
1. Permissions are granted
2. Using correct EventKit APIs
3. Proper error handling with `RemError` types

### Debugging Tips

1. **Add print statements** in key areas:
   ```rust
   println!("Key pressed: {:?}", key.code);
   println!("Action generated: {:?}", action);
   ```

2. **Use the debug builds** for more information:
   ```bash
   make debug
   ```

3. **Check the TUI state**:
   ```rust
   println!("Current view: {:?}", self.current_view);
   println!("Selected index: {}", self.selected_index);
   ```

4. **Verify Swift action processing**:
   ```swift
   print("Processing action: \(action)")
   ```

## Best Practices

### 1. Naming Conventions

- **Actions**: Use descriptive names like `DeleteReminder`, `ToggleReminder`
- **Parameters**: Use clear names like `reminder_id`, `list_id`
- **Functions**: Use verb phrases like `deleteReminder`, `toggleCompletion`

### 2. Error Handling

- Always use appropriate `RemError` types
- Provide descriptive error messages
- Handle EventKit failures gracefully

### 3. User Experience

- Provide immediate feedback for actions
- Use consistent key bindings (vim-style preferred)
- Show helpful instructions in the UI
- Consider confirmation for destructive actions

### 4. Code Organization

- Keep action definitions together in `lib.rs`
- Group related functionality in the same file
- Use clear comments for complex key sequences
- Document public APIs

### 5. Testing

- Test both happy path and error cases
- Verify key sequences work correctly
- Test with empty lists and edge cases
- Ensure proper cleanup of resources

This guide should help you understand how to add new functionality to Rem through all layers of the architecture. Each change follows the same pattern: define in Rust, update UniFFI interface, rebuild bindings, implement Swift handling, and test thoroughly.