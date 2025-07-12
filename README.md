# Rem - Apple Reminders TUI

A fast, beautiful Terminal User Interface (TUI) for Apple Reminders built with Rust. Rem provides a keyboard-driven interface to view, navigate, and manage your Apple Reminders directly from the terminal.

[![CI](https://github.com//rem/workflows/CI/badge.svg)](https://github.com//rem/actions)

## Features

- 🚀 **Fast & Lightweight**: Built with Rust for maximum performance
- 📱 **Real Apple Reminders Data**: Direct integration with macOS Reminders app
- ⌨️ **Vim-style Navigation**: Intuitive keyboard shortcuts (j/k, arrow keys)
- 🎨 **Beautiful UI**: Modern terminal interface with colors, emojis, and rounded borders
- ✅ **Interactive**: Toggle reminder completion status
- 🔍 **Live Data**: Real-time access to your actual reminders and lists
- 🛡️ **Secure**: Uses Apple's EventKit framework with proper permission handling

## Screenshots

```
┌─────────────────── 📝 Your Reminder Lists ───────────────────┐
│                                                              │
│  ▶ ●  Today                                                  │
│      9306 reminders                                          │
│                                                              │
│    ●  House                                                  │
│      51 reminders                                            │
│                                                              │
│    ●  Our Things                                             │
│      12 reminders                                            │
│                                                              │
└──────────────────────────────────────────────────────────────┘
┌───────────────────── Controls ─────────────────────┐
│        ↑↓ or j/k navigate  ⏎ select  q quit        │
└─────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- macOS (required for Apple Reminders integration)
- Rust 1.70+ 
- Xcode Command Line Tools

### From Source

```bash
git clone https://github.com/yourusername/rem.git
cd rem
cargo build --release
./target/release/rem
```

### Using Cargo

```bash
cargo install rem
```

## Usage

### Basic Usage

```bash
# Start the application
rem

# Run with debug logging
DEBUG=true rem

# Run with custom tick/frame rates
rem --tick-rate 2.0 --frame-rate 30.0
```

### Navigation

**Lists View:**
- `j`/`k` or `↑`/`↓` - Navigate between lists
- `Enter` - Open selected list
- `q` - Quit application

**Reminders View:**
- `j`/`k` or `↑`/`↓` - Navigate between reminders
- `Space` - Toggle reminder completion
- `Esc` - Go back to lists
- `q` - Quit application

### Permissions

On first run, Rem will request permission to access your Reminders. This is required for the app to function and uses Apple's standard permission system.

## Architecture

### Overview

Rem follows a component-based architecture built on the Ratatui framework, with an event-driven design that separates concerns between UI rendering, data fetching, and user interaction.

```
┌─────────────────────────────────────────────────────────┐
│                    Terminal UI (Ratatui)                │
├─────────────────────────────────────────────────────────┤
│  Components Layer                                       │
│  ┌──────────────┐ ┌──────────────┐ ┌─────────────────┐  │
│  │ Permission   │ │    Lists     │ │   Reminders     │  │
│  │ Component    │ │  Component   │ │   Component     │  │
│  └──────────────┘ └──────────────┘ └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                   Event System                          │
│              (Actions & Message Passing)                │
├─────────────────────────────────────────────────────────┤
│                  Data Integration                       │
│  ┌──────────────┐              ┌──────────────────────┐ │
│  │   EventKit   │              │     AppleScript      │ │
│  │  Framework   │──────────────│      Bridge          │ │
│  │ (Objective-C)│              │   (osascript CLI)    │ │
│  └──────────────┘              └──────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                 Apple Reminders                         │
│                    (macOS App)                          │
└─────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Application Core (`src/main.rs`, `src/app.rs`)

**`main.rs`**
- Entry point with CLI argument parsing
- Error handling and logging initialization
- Terminal environment validation

**`app.rs`**
- Main application loop and state management
- Component lifecycle management
- Event routing and action dispatching
- Async runtime coordination

#### 2. Component System (`src/components/`)

All UI components implement the `Component` trait which provides:

```rust
pub trait Component {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()>;
    fn register_config_handler(&mut self, config: Config) -> Result<()>;
    fn update(&mut self, action: Action) -> Result<Option<Action>>;
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>>;
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>>;
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()>;
}
```

**Permission Component** (`permission.rs`)
- Handles macOS permission requests for Reminders access
- Manages EventKit framework initialization
- Displays permission status and instructions to user
- Provides secure access to Apple's EventKit APIs

**Lists Component** (`lists.rs`)
- Displays all available reminder lists
- Shows list colors, names, and reminder counts
- Handles list selection and navigation
- Implements beautiful card-based UI design

**Reminders Component** (`reminders.rs`)
- Shows individual reminders within a selected list
- Displays completion status, priority, notes, and due dates
- Allows toggling reminder completion
- Supports vim-style navigation

#### 3. Event System (`src/action.rs`)

The application uses a message-passing architecture with typed actions:

```rust
pub enum Action {
    Quit,
    Suspend,
    Resume,
    CheckPermissions,
    RequestPermissions,
    LoadLists,
    LoadReminders(String),
    SelectList(String),
    Back,
    Error(String),
}
```

**Event Flow:**
1. User input (keyboard/mouse) generates events
2. Components handle events and optionally produce actions
3. Actions are sent through message channels
4. App receives actions and updates state accordingly
5. Components re-render based on new state

#### 4. Data Integration (`src/eventkit.rs`)

**Hybrid Approach**: Combines EventKit framework with AppleScript for optimal reliability

**EventKit Framework Integration:**
- Uses Objective-C bindings via `objc` crate
- Handles permission management
- Fetches list metadata (names, IDs, colors)
- Provides synchronous operations for list enumeration

**AppleScript Bridge:**
- Executes AppleScript via `osascript` command
- Retrieves actual reminder data and counts
- Bypasses EventKit's async completion block issues
- Ensures reliable data access in CLI environments

```rust
// Example: Getting reminder count via AppleScript
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
```

### Technical Methodology

#### 1. Rust + Ratatui Foundation

**Why Rust:**
- Memory safety without garbage collection
- Excellent performance for TUI applications
- Strong ecosystem for terminal applications
- Cross-platform compatibility

**Why Ratatui:**
- Immediate mode GUI perfect for terminal interfaces
- Efficient rendering with minimal flicker
- Rich widget set with customizable styling
- Event-driven architecture support

#### 2. EventKit Integration Challenges & Solutions

**Challenge**: EventKit's async completion blocks don't work reliably in CLI applications because they expect a running NSRunLoop.

**Solution**: Hybrid approach combining EventKit and AppleScript:
- **EventKit**: Used for synchronous operations (permission checking, list enumeration)
- **AppleScript**: Used for data fetching (reminder counts, reminder content)

**Implementation Details:**
```rust
// EventKit for list enumeration (synchronous)
let calendars: *mut Object = msg_send![self.event_store, calendarsForEntityType: 1i64];

// AppleScript for data fetching (reliable)
let script = format!(r#"tell application "Reminders" to count of reminders in list "{}""#, list_name);
let output = Command::new("osascript").arg("-e").arg(&script).output()?;
```

#### 3. Component-Based Architecture

**Benefits:**
- **Separation of Concerns**: Each component handles specific functionality
- **Reusability**: Components can be easily tested and modified
- **State Management**: Clear data flow between components
- **Event Handling**: Decoupled event processing

**Design Pattern:**
```
User Input → Event → Component Handler → Action → App State Update → Re-render
```

#### 4. Error Handling Strategy

**Multi-layered Approach:**
- **Result Types**: All operations return `Result<T, E>` for explicit error handling
- **Color Eyre**: Beautiful error reporting with context and suggestions
- **Graceful Degradation**: App continues functioning even if some operations fail
- **User Feedback**: Clear error messages displayed in UI

#### 5. Performance Optimizations

**Efficient Rendering:**
- Only re-render when state changes
- Minimal string allocations during UI updates
- Stateful widgets for list navigation

**Smart Data Fetching:**
- Cache reminder counts to avoid repeated AppleScript calls
- Lazy loading of reminder details
- Debounced user input processing

**Memory Management:**
- Rust's ownership system prevents memory leaks
- Careful management of Objective-C objects
- Minimal heap allocations in render loops

#### 6. Platform Integration

**macOS-Specific Features:**
- Native EventKit framework integration
- Proper permission handling following Apple guidelines
- AppleScript integration for reliable data access
- Respects system appearance and accessibility settings

**Permission Model:**
1. Check existing permissions on startup
2. Request permissions if needed using EventKit APIs
3. Handle all permission states (authorized, denied, restricted, not determined)
4. Provide clear user guidance for permission issues

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --locked --all-features --workspace

# Format code
cargo fmt --all --check

# Lint code
cargo clippy --all-targets --all-features --workspace -- -D warnings
```

### Debug Logging

Enable detailed debug output to see internal operations:

```bash
DEBUG=true cargo run
```

This will show:
- Permission checking process
- EventKit framework operations
- AppleScript execution and results
- Component lifecycle events
- Data loading progress

### Project Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Main application loop
├── tui.rs               # Terminal interface wrapper
├── action.rs            # Action definitions
├── config.rs            # Configuration management
├── eventkit.rs          # Apple Reminders integration
├── components/
│   ├── mod.rs           # Component trait definition
│   ├── permission.rs    # Permission handling UI
│   ├── lists.rs         # Reminder lists view
│   └── reminders.rs     # Individual reminders view
├── cli.rs               # Command line parsing
├── errors.rs            # Error handling setup
└── logging.rs           # Logging configuration
```

### Key Dependencies

- **`ratatui`**: Terminal UI framework
- **`crossterm`**: Cross-platform terminal handling
- **`tokio`**: Async runtime
- **`objc`**: Objective-C runtime bindings
- **`color-eyre`**: Error handling and reporting
- **`clap`**: Command line argument parsing
- **`serde`**: Serialization for configuration

## Troubleshooting

### Permission Issues

If the app can't access your reminders:

1. **Check System Preferences**:
   - Open System Preferences → Security & Privacy → Privacy
   - Select "Reminders" from the list
   - Ensure your terminal app is checked

2. **Reset Permissions** (if needed):
   ```bash
   tccutil reset Reminders
   ```

3. **Terminal App Permissions**:
   - Make sure you're running from a terminal that has Reminders access
   - Some terminal apps may need individual permission grants

### Performance Issues

- **Slow startup**: Usually due to permission dialogs or large reminder counts
- **UI lag**: Try reducing tick/frame rates: `rem --tick-rate 1.0 --frame-rate 20.0`
- **Memory usage**: Enable debug logging to identify bottlenecks: `DEBUG=true rem`

### Compatibility

- **macOS Version**: Requires macOS 10.14+ for EventKit framework
- **Terminal Compatibility**: Works with all modern terminal emulators
- **Rust Version**: Requires Rust 1.70+ for latest dependency features

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Run the test suite (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run Clippy lints (`cargo clippy`)
- Add documentation for public APIs
- Include tests for new functionality
- Follow the existing component architecture

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) - excellent TUI framework
- Inspired by terminal-based productivity tools
- Thanks to the Rust community for amazing crates and documentation