# Architecture Documentation

## Overview

Rem uses a **Swift wrapper + Rust core** architecture that separates system integration concerns from TUI implementation. This design provides the best of both worlds: native macOS integration through Swift/EventKit and high-performance terminal UI through Rust/Ratatui.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          Swift Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │    main.swift   │  │RemindersService │  │PermissionManager│ │
│  │ (Entry Point)   │  │   (EventKit)    │  │ (Native Dialogs)│ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                         UniFFI Bridge                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Type-Safe Rust ↔ Swift Communication                  │   │
│  │  • Automatic serialization/deserialization             │   │
│  │  • Memory management                                    │   │
│  │  • Error propagation                                    │   │
│  │  • Checksum validation                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                          Rust Core                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    lib.rs (API)                         │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐    │   │
│  │  │ start_tui() │ │set_reminders│ │render_reminders │    │   │
│  │  │             │ │    ()       │ │    _view()      │    │   │
│  │  └─────────────┘ └─────────────┘ └─────────────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  TUI Application                        │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │                  Ratatui Engine                     │ │   │
│  │  │  ┌───────────┐ ┌──────────┐ ┌─────────────────┐    │ │   │
│  │  │  │   Lists   │ │Reminders │ │   Navigation    │    │ │   │
│  │  │  │ Component │ │Component │ │   & Events      │    │ │   │
│  │  │  └───────────┘ └──────────┘ └─────────────────┘    │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Apple Reminders                           │
│                        (macOS App)                             │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### Swift Layer

#### RemTUI/Sources/main.swift
- **Purpose**: Application entry point and orchestration
- **Responsibilities**:
  - Initialize permission manager
  - Load reminders data via RemindersService
  - Call Rust TUI with loaded data
  - Handle returned actions from TUI
  - Process user interactions (future: create, update, delete)

#### RemTUIKit/Sources/RemTUIKit/PermissionManager.swift
- **Purpose**: Native macOS permission handling
- **Responsibilities**:
  - Check current Reminders access status
  - Request permissions with native dialogs
  - Provide user guidance for permission setup
  - Handle permission-related errors gracefully

#### RemTUIKit/Sources/RemTUIKit/RemindersService.swift
- **Purpose**: Fast EventKit integration
- **Responsibilities**:
  - Load reminder lists concurrently
  - Fetch reminders for specific lists
  - Convert EventKit objects to Rust-compatible types
  - Handle EventKit errors and edge cases
  - Provide real-time data access

### UniFFI Bridge Layer

#### rust-core/src/rem_core.udl
- **Purpose**: Interface definition between Rust and Swift
- **Defines**:
  - Data structures (ReminderList, Reminder, TuiAction)
  - Error types (RemError)
  - Function signatures for Rust exports
  - Type mappings and serialization rules

#### Generated Bindings
- **RemCore.swift**: Auto-generated Swift interface to Rust
- **rem_coreFFI.h**: C header for FFI calls
- **Checksum validation**: Ensures library/binding compatibility

### Rust Core Layer

#### rust-core/src/lib.rs
- **Purpose**: Main Rust API surface
- **Exports**:
  - `start_tui(lists: Vec<ReminderList>) -> Result<Vec<TuiAction>, RemError>`
  - `render_reminders_view(reminders: Vec<Reminder>) -> Result<Vec<TuiAction>, RemError>`
  - `set_reminders(reminders: Vec<Reminder>) -> Result<(), RemError>`
- **Features**:
  - Global TUI state management
  - Thread-safe access via Mutex
  - Error conversion and propagation

#### rust-core/src/tui/app.rs
- **Purpose**: Core TUI application logic
- **Components**:
  - Terminal initialization and cleanup
  - Event loop management
  - View state management (Lists vs Reminders)
  - Key event handling
  - Screen rendering coordination

#### rust-core/src/tui/components.rs & events.rs
- **Purpose**: UI components and event handling
- **Features**:
  - Ratatui widget integration
  - Custom styling and layouts
  - Navigation state management
  - User interaction processing

## Data Flow

### Startup Sequence

1. **Swift Entry**: `main.swift` starts application
2. **Permission Check**: `PermissionManager` verifies Reminders access
3. **Data Loading**: `RemindersService` loads lists via EventKit
4. **TUI Launch**: Swift calls `start_tui()` with loaded data
5. **TUI Initialization**: Rust creates TUI app with reminder lists
6. **Event Loop**: TUI handles user input and renders interface
7. **Action Return**: User actions returned to Swift for processing

### User Interaction Flow

```
User Input → Rust TUI → TuiAction → Swift Handler → EventKit Update → Reload Data
     ↑                                                                      │
     └──────────────────── Updated Interface ←─────────────────────────────┘
```

### Error Handling

- **Rust Layer**: All errors converted to `RemError` enum
- **UniFFI**: Automatic error propagation across FFI boundary
- **Swift Layer**: Native error handling with user-friendly messages
- **Graceful Degradation**: Partial failures don't crash the application

## Build Process

### Compilation Flow

1. **Rust Compilation**: `cargo build --release` creates `librem_core.dylib`
2. **UniFFI Generation**: `uniffi-bindgen` creates Swift bindings from UDL
3. **Swift Package Build**: Swift Package Manager links against Rust library
4. **Executable Creation**: Final binary includes all components

### Key Build Commands

```bash
make build-rust      # Step 1: Compile Rust core
make build-uniffi    # Step 2: Generate UniFFI bindings + copy library
make build-swift     # Step 3: Build Swift package and executable
make build          # All steps combined
```

### UniFFI Checksum Management

UniFFI generates checksums to ensure the Rust library matches the Swift bindings:

- **Generated checksums**: Based on function signatures and types
- **Validation**: Checked at runtime during initialization
- **Common issue**: Rebuilding Rust changes checksums, requiring binding regeneration
- **Solution**: Use `make run-direct` to avoid unnecessary rebuilds

## Performance Characteristics

### Memory Usage
- **Rust Core**: Efficient memory management with ownership system
- **Swift Layer**: ARC-managed objects with minimal retention cycles
- **Data Transfer**: Zero-copy data sharing where possible via UniFFI

### Startup Performance
- **Cold Start**: <1 second (vs 18+ seconds with AppleScript)
- **Permission Check**: <100ms with native APIs
- **Data Loading**: <500ms for typical reminder collections
- **TUI Initialization**: <50ms for terminal setup

### Runtime Performance
- **Navigation**: Immediate response (<16ms for 60fps)
- **Data Updates**: <200ms round-trip for EventKit updates
- **Memory Footprint**: <50MB total for large reminder collections

## Security Considerations

### Permission Model
- **Explicit Consent**: Native macOS permission dialogs
- **Minimal Access**: Only Reminders data, no other personal information
- **Sandboxing**: Future App Store compatibility considerations

### Data Handling
- **Local Only**: No network access or data transmission
- **Memory Safety**: Rust's ownership system prevents buffer overflows
- **Type Safety**: UniFFI ensures data integrity across language boundaries

## Development Guidelines

### Adding New Features

1. **Define Interface**: Update `rem_core.udl` with new types/functions
2. **Implement Rust**: Add functionality to appropriate Rust modules
3. **Update Swift**: Modify Swift layer to use new Rust functions
4. **Test Integration**: Verify end-to-end functionality
5. **Update Documentation**: Document new capabilities

### Debugging

- **Swift Layer**: Use Xcode debugger or `print()` statements
- **Rust Layer**: Use `RUST_LOG=debug` environment variable
- **UniFFI Issues**: Check generated binding checksums
- **Performance**: Use `cargo flamegraph` for Rust profiling

### Testing Strategy

- **Unit Tests**: Each layer tested independently
- **Integration Tests**: Full Swift→Rust→Swift round trips
- **Performance Tests**: Benchmark critical paths
- **Manual Testing**: Real-world usage scenarios

## Future Architecture Evolution

### Planned Enhancements

1. **Bidirectional Updates**: Rust can trigger Swift data reloads
2. **Streaming Data**: Real-time reminder updates from EventKit
3. **Plugin System**: Modular extensions for new features
4. **Cross-Platform**: Windows/Linux support via alternative Swift layer

### Scalability Considerations

- **Large Data Sets**: Pagination and lazy loading strategies
- **Memory Management**: Streaming and caching optimizations
- **Performance Monitoring**: Built-in metrics and profiling
- **Error Recovery**: Robust error handling and user guidance