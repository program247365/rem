# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rem` is a fast TUI (Terminal User Interface) for Apple Reminders built with Rust. It uses the Ratatui library for terminal UI rendering and follows an event-driven architecture with async components.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application
cargo run

# Run with custom tick/frame rates
cargo run -- --tick-rate 2.0 --frame-rate 30.0
```

### Testing and Quality
```bash
# Run tests
cargo test --locked --all-features --workspace

# Format code
cargo fmt --all --check

# Run clippy linting
cargo clippy --all-targets --all-features --workspace -- -D warnings

# Generate documentation
cargo doc --no-deps --document-private-items --all-features --workspace --examples
```

## Architecture

### Core Components

The application follows a component-based architecture:

- **`main.rs`**: Entry point that initializes error handling, logging, and the main app
- **`app.rs`**: Main application struct that manages the event loop, components, and state
- **`tui.rs`**: Terminal interface wrapper that handles crossterm events and rendering
- **`components/`**: UI components that implement the `Component` trait

### Component System

All UI components implement the `Component` trait (`src/components.rs`) which provides:
- **Action handling**: `register_action_handler()` for sending actions
- **Configuration**: `register_config_handler()` for receiving config
- **Event handling**: `handle_events()`, `handle_key_event()`, `handle_mouse_event()`
- **State updates**: `update()` method for processing actions
- **Rendering**: `draw()` method for UI rendering

Current components:
- `Home`: Main interface component (currently displays "hello world")
- `FpsCounter`: Debug component for showing frame rate

### Event System

The application uses a message-passing architecture:
- **Events** (`tui::Event`): Raw terminal events (key presses, mouse, resize, etc.)
- **Actions** (`action::Action`): High-level application commands (Quit, Suspend, etc.)
- Components can send actions via `UnboundedSender<Action>` channels

### Configuration

Configuration is handled through:
- **Default config**: Embedded in `src/config.rs` from `.config/config.json5`
- **User config**: Loaded from standard OS config directories
- **Key bindings**: Configurable per-mode key mappings
- **Styles**: Configurable UI styling

Supported config formats: JSON5, JSON, YAML, TOML, INI

### Build System

Uses `vergen-gix` for build-time information (git hash, build date, etc.) via `build.rs`.

## Key Files

- `src/main.rs`: Application entry point
- `src/app.rs`: Main application loop and component management
- `src/tui.rs`: Terminal interface and event handling
- `src/components.rs`: Component trait definition
- `src/action.rs`: Action enum definitions
- `src/config.rs`: Configuration system with key binding parsing
- `src/cli.rs`: Command line argument parsing
- `.config/config.json5`: Default configuration with key bindings

## Application Modes

Currently supports one mode:
- **Home**: Default mode with basic quit/suspend key bindings (q, Ctrl-d, Ctrl-c, Ctrl-z)

## Dependencies

Key dependencies:
- `ratatui`: Terminal UI framework
- `crossterm`: Cross-platform terminal handling
- `tokio`: Async runtime
- `clap`: Command line parsing
- `config`: Configuration management
- `color-eyre`: Error handling
- `tracing`: Logging