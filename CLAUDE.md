# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rem` is a fast TUI (Terminal User Interface) for Apple Reminders built with a modern **Swift wrapper + Rust core** architecture. It uses native EventKit integration for blazing-fast performance and UniFFI for type-safe cross-language communication.

## Architecture

The project has been migrated from a monolithic Rust architecture to a layered approach:

### Swift Layer (System Integration)
- **RemTUIKit**: Swift package providing native macOS integration
- **RemTUI**: Swift executable serving as the entry point
- **Native EventKit**: Fast, reliable access to Reminders data
- **Permission Management**: Native macOS permission dialogs

### Rust Layer (TUI Core)  
- **rust-core**: Rust library containing the TUI implementation
- **Ratatui**: High-performance terminal interface
- **Event System**: Keyboard navigation and user interaction
- **Component Architecture**: Modular UI components

### UniFFI Bridge
- **Type-safe Communication**: Automatic binding generation
- **rem_core.udl**: Interface definition language
- **Generated Bindings**: Swift stubs for Rust functions

## Development Commands

### Quick Start
```bash
# Setup development environment
make setup

# Build everything
make build

# Run the application  
make run

# Development mode with auto-rebuild
make dev
```

### Building Components

```bash
# Build Rust core library
make build-rust

# Build Swift package and executable
make build-swift

# Generate UniFFI bindings
make build-uniffi

# Build everything in debug mode
make build-debug
```

### Testing and Quality

```bash
# Run all tests
make test

# Test individual components
make test-rust          # Rust core tests
make test-swift         # Swift package tests
make test-integration   # Integration tests
make test-migration     # Architecture validation

# Code quality checks
make check              # All checks
make check-rust         # Rust formatting, linting, compilation
make check-swift        # Swift compilation

# Format and lint code
make fmt                # Format all code
make fmt-rust           # Format Rust only
make fmt-swift          # Format Swift only
make lint               # Run all linters
```

### Debugging and Development

```bash
# Run with debug logging
make debug

# Debug specific components
make debug-rust         # Debug Rust core
make debug-swift        # Debug Swift components
make debug-build        # Debug build process

# Development workflow
make dev                # Auto-rebuild on changes
make run-debug          # Run debug build
make quick-run          # Quick build and run

# Troubleshooting
make debug-permissions  # Debug permission issues
make check-permissions  # Check current permissions
make check-system       # Verify system requirements
```

### Performance and Profiling

```bash
# Run benchmarks
make benchmark          # Performance comparison
make profile            # Application profiling

# Clean builds
make clean              # Clean build artifacts
make clean-all          # Deep clean everything
```

## Project Structure

```
RemTUI/                           # New Architecture
├── RemTUIKit/                    # Swift Package (System Integration)
│   ├── Package.swift            # Swift package configuration
│   ├── Sources/RemTUIKit/
│   │   ├── RemindersService.swift    # Native EventKit wrapper
│   │   ├── PermissionManager.swift   # Native permission handling
│   │   └── [Generated UniFFI files]  # Rust bindings (auto-generated)
│   └── Tests/RemTUIKitTests/
├── rust-core/                   # Rust Library (TUI Core)
│   ├── Cargo.toml              # Rust dependencies
│   ├── src/
│   │   ├── lib.rs              # UniFFI exports & main interface
│   │   ├── rem_core.udl        # UniFFI interface definition
│   │   ├── types.rs            # Shared data structures
│   │   └── tui/               # Ratatui implementation
│   │       ├── mod.rs         # Module exports
│   │       ├── app.rs         # Main TUI application logic
│   │       ├── components.rs  # UI component definitions
│   │       └── events.rs      # Event handling system
├── RemTUI/                     # Swift Executable
│   ├── Package.swift          # Executable package config
│   ├── Sources/main.swift     # Application entry point
│   └── Info.plist            # Permissions & metadata
├── build.sh                   # Build coordination script
├── test-migration.sh          # Migration validation
├── Makefile                   # Complete development workflow
├── MIGRATION.md              # Architecture migration guide
├── CLAUDE.md                 # This file
└── README.md                 # User-facing documentation
```

## Key Components

### Swift Layer Files

**RemTUIKit/Sources/RemTUIKit/RemindersService.swift**
- Native EventKit integration for fast data access
- Async/await for concurrent list and reminder loading
- Robust error handling with typed exceptions
- Color extraction from calendar metadata

**RemTUIKit/Sources/RemTUIKit/PermissionManager.swift**
- Native permission request dialogs (no manual setup!)
- Permission status checking and guidance
- Automatic permission flow handling
- User-friendly error messages

**RemTUI/Sources/main.swift**
- Application entry point and Swift-Rust coordination
- Permission checking and data loading workflow
- Action processing loop between Swift and Rust
- Error handling and user feedback

### Rust Layer Files

**rust-core/src/lib.rs**
- UniFFI function exports (`start_tui`, `render_reminders_view`, etc.)
- Global TUI state management
- Type definitions (`ReminderList`, `Reminder`, `TUIAction`, `RemError`)
- Error handling with structured types

**rust-core/src/rem_core.udl**
- UniFFI interface definition language
- Type-safe function signatures
- Error enum definitions
- Data structure specifications

**rust-core/src/tui/app.rs**
- Core TUI application logic with Ratatui
- List and reminder rendering with beautiful styling
- Keyboard event handling (vim-style navigation)
- Terminal setup/teardown and render loops

### Build System Files

**Makefile**
- Complete development workflow automation
- Component-specific build targets
- Testing, debugging, and profiling commands
- Color-coded output and helpful error messages

**build.sh**
- Coordinates Rust compilation and UniFFI binding generation
- Copies dynamic libraries to correct locations
- Handles dependency resolution

## Data Flow

1. **Swift Entry Point** (`main.swift`): Handles permissions and loads data via native EventKit
2. **Native Integration** (`RemindersService`): Fast, reliable access to Reminders data
3. **UniFFI Bridge**: Type-safe data conversion between Swift and Rust types
4. **Rust TUI** (`tui/app.rs`): High-performance terminal interface and user interaction
5. **Action Processing**: Swift receives actions from Rust and updates data accordingly

## Development Workflow

### First-time Setup
1. `make setup` - Install dependencies and tools
2. `make build` - Build all components with UniFFI bindings  
3. `make test` - Validate everything works
4. `make run` - Start the application

### Daily Development
1. `make dev` - Start development mode with auto-rebuild
2. Edit code in your preferred editor
3. Changes trigger automatic rebuild and restart
4. `make test` - Run tests before committing
5. `make check` - Final quality checks

### Debugging Issues
1. `make debug` - Run with detailed logging
2. `make debug-rust` - Debug Rust components specifically
3. `make debug-swift` - Debug Swift components specifically  
4. `make debug-permissions` - Troubleshoot permission issues
5. `make clean-all && make build` - Clean rebuild if needed

### Performance Analysis
1. `make benchmark` - Compare performance metrics
2. `make profile` - Profile application with Instruments
3. Monitor cold start times and memory usage

## Migration Notes

This project was migrated from a monolithic Rust architecture to the current Swift wrapper + Rust core design. Key improvements:

### Performance Gains
- **Cold Start**: 18+ seconds → <1 second (eliminated AppleScript delays)
- **List Loading**: 2-3 seconds → <500ms (concurrent EventKit calls)
- **Reminder Fetch**: 1-2 seconds/list → <200ms/list (native APIs)

### User Experience Improvements  
- **Permissions**: Manual setup → Native dialogs
- **Error Handling**: Generic messages → Contextual guidance
- **Feature Access**: Basic reminders → Full EventKit features

### Developer Experience Improvements
- **Type Safety**: Manual FFI → UniFFI-generated bindings
- **Debugging**: Limited tools → Component-specific debugging
- **Testing**: Single test suite → Layered testing strategy
- **Build System**: Basic Cargo → Comprehensive Makefile workflow

## Important Notes

1. **Requires macOS 14.0+** for full EventKit access APIs
2. **UniFFI bindings** are auto-generated - don't edit the generated Swift files
3. **Rust dynamic library** must be copied to Swift package for linking
4. **Permission handling** is automatic - no manual TCC.db editing needed
5. **Development mode** watches all source directories for changes

## Common Development Tasks

### Adding New Rust Functions
1. Add function signature to `rust-core/src/rem_core.udl`
2. Implement function in `rust-core/src/lib.rs` with `#[uniffi::export]`
3. Run `make build-uniffi` to regenerate Swift bindings
4. Use new function from Swift in `RemTUI/Sources/main.swift`

### Adding New Swift Features
1. Implement in `RemTUIKit/Sources/RemTUIKit/`
2. Add to Package.swift if needed
3. Test with `make test-swift`
4. Use from main app in `RemTUI/Sources/main.swift`

### Debugging Build Issues
1. `make clean-all` - Clean everything
2. `make check-system` - Verify requirements
3. `make debug-build` - Verbose build output
4. Check individual components with `make check-rust` and `make check-swift`

### Performance Optimization
1. `make benchmark` - Establish baseline
2. Profile with `make profile` 
3. Optimize hot paths in Rust or Swift
4. Validate improvements with `make benchmark`

This architecture provides excellent separation of concerns while maintaining high performance and native macOS integration.