# Developer Guide - Rem TUI

A comprehensive guide for developers working on the Rem Apple Reminders TUI application.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure Deep Dive](#project-structure-deep-dive)
- [Development Workflows](#development-workflows)
- [Testing Strategy](#testing-strategy)
- [Debugging Guide](#debugging-guide)
- [Performance Optimization](#performance-optimization)
- [Contributing Guidelines](#contributing-guidelines)
- [Troubleshooting](#troubleshooting)

## Architecture Overview

Rem uses a modern **Swift wrapper + Rust core** architecture that provides the best of both worlds:

### Why This Architecture?

1. **Swift Layer**: Native macOS integration, automatic permissions, fast EventKit access
2. **Rust Core**: High-performance TUI, memory safety, excellent terminal libraries
3. **UniFFI Bridge**: Type-safe communication, automatic binding generation, zero-copy data transfer

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   macOS System                          │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │   Reminders     │    │      EventKit Framework     │ │
│  │     App         │    │     (System Permissions)    │ │
│  └─────────────────┘    └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                              ▲
                              │ Native API Calls
                              ▼
┌─────────────────────────────────────────────────────────┐
│                Swift Wrapper Layer                      │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │RemindersService │    │    PermissionManager        │ │
│  │ • EventKit APIs │    │ • Native Dialogs            │ │
│  │ • Data Loading  │    │ • Status Checking           │ │
│  │ • Error Handling│    │ • User Guidance             │ │
│  └─────────────────┘    └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                              ▲
                              │ Type-Safe FFI
                              ▼
┌─────────────────────────────────────────────────────────┐
│                   UniFFI Bridge                         │
│  • Automatic binding generation                         │
│  • Type safety across language boundary                 │
│  • Zero-copy data structures where possible             │
│  • Error propagation and handling                       │
└─────────────────────────────────────────────────────────┘
                              ▲
                              │ Generated Bindings
                              ▼
┌─────────────────────────────────────────────────────────┐
│                  Rust Core Layer                        │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                Ratatui TUI Engine                   │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐│ │
│  │  │   Lists     │ │ Reminders   │ │   Navigation    ││ │
│  │  │ Component   │ │ Component   │ │   & Events      ││ │
│  │  └─────────────┘ └─────────────┘ └─────────────────┘│ │
│  │                                                     │ │
│  │  ┌─────────────────────────────────────────────────┐│ │
│  │  │           Terminal Interface                    ││ │
│  │  │ • Keyboard handling • Rendering • State mgmt   ││ │
│  │  └─────────────────────────────────────────────────┘│ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

1. **User Action**: User presses key in terminal
2. **Rust Processing**: TUI captures event, processes it, generates action
3. **Swift Integration**: Action sent to Swift layer via UniFFI
4. **System Call**: Swift makes native EventKit API calls
5. **Data Return**: Updated data flows back through UniFFI to Rust
6. **UI Update**: Rust TUI re-renders with new data

## Development Environment Setup

### Prerequisites

- **macOS 14.0+** (required for EventKit full access APIs)
- **Xcode Command Line Tools** (`xcode-select --install`)
- **Rust 1.70+** with Cargo (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Swift 5.9+** (included with Xcode CLT)

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/rem.git
cd rem

# Setup development environment
make setup

# Verify installation
make check-system

# Build everything
make build

# Run tests
make test

# Start the application
make run
```

### Development Tools (Optional)

```bash
# Enhanced development experience
make install-dev

# Individual tool installation
cargo install cargo-watch     # Auto-rebuild on file changes
cargo install cargo-nextest   # Faster test runner
cargo install cargo-expand    # Macro expansion debugging
cargo install uniffi-bindgen  # UniFFI binding generation

# Swift tools
brew install swiftformat      # Swift code formatting
brew install swiftlint        # Swift linting
```

## Project Structure Deep Dive

### Swift Layer (`RemTUIKit/` and `RemTUI/`)

#### RemTUIKit Package Structure
```
RemTUIKit/
├── Package.swift                 # Swift Package Manager configuration
├── Sources/RemTUIKit/
│   ├── RemindersService.swift    # EventKit integration
│   ├── PermissionManager.swift   # Permission handling
│   └── [Generated Files]         # UniFFI-generated bindings
│       ├── rem_core.swift        # Swift wrapper functions
│       ├── rem_coreFFI.h         # C header for FFI
│       └── librem_core.dylib     # Rust library
└── Tests/RemTUIKitTests/
    └── RemTUIKitTests.swift      # Unit tests
```

#### Key Swift Files

**RemindersService.swift**
```swift
public class RemindersService: ObservableObject {
    private let eventStore = EKEventStore()
    
    // Core functionality:
    // - Permission management with native dialogs
    // - Concurrent list and reminder fetching
    // - Real-time data updates
    // - Error handling with context
}
```

**PermissionManager.swift**
```swift
public class PermissionManager {
    // Handles all permission-related functionality:
    // - Status checking (authorized, denied, etc.)
    // - Native permission request dialogs
    // - User guidance and troubleshooting
    // - Integration with system settings
}
```

**main.swift**
```swift
@main
struct RemTUIApp {
    static func main() async {
        // Application flow:
        // 1. Check/request permissions
        // 2. Load reminder data
        // 3. Start Rust TUI
        // 4. Process user actions
        // 5. Update data and restart TUI loop
    }
}
```

### Rust Layer (`rust-core/`)

#### Rust Core Structure
```
rust-core/
├── Cargo.toml                   # Dependencies and build configuration
├── src/
│   ├── lib.rs                   # UniFFI exports and main interface
│   ├── rem_core.udl             # UniFFI interface definition
│   ├── types.rs                 # Shared data types
│   └── tui/                     # TUI implementation
│       ├── mod.rs               # Module exports
│       ├── app.rs               # Main TUI application
│       ├── components.rs        # UI component definitions
│       └── events.rs            # Event handling system
├── build.rs                     # Build script for UniFFI
└── uniffi.toml                  # UniFFI configuration
```

#### Key Rust Files

**lib.rs**
```rust
// UniFFI function exports
#[uniffi::export]
pub fn start_tui(lists: Vec<ReminderList>) -> Result<Vec<TUIAction>, RemError> {
    // Initialize TUI with reminder lists
    // Run main event loop
    // Return actions for Swift to process
}

// Data type definitions
#[derive(uniffi::Record)]
pub struct ReminderList { /* ... */ }

// Error handling
#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum RemError { /* ... */ }
```

**rem_core.udl**
```idl
// UniFFI interface definition
namespace rem_core {
    [Throws=RemError]
    sequence<TUIAction> start_tui(sequence<ReminderList> lists);
};

// Type definitions must match Rust exactly
dictionary ReminderList {
    string id;
    string name;
    string color;
    u32 count;
};
```

**tui/app.rs**
```rust
pub struct TUIApp {
    // Core TUI state and logic
    // - Terminal setup/teardown
    // - Event handling (keyboard, mouse)
    // - Rendering with Ratatui
    // - State management
}
```

## Development Workflows

### Daily Development Workflow

1. **Start Development Mode**
   ```bash
   make dev
   # Watches for file changes and auto-rebuilds
   ```

2. **Make Changes**
   - Edit Swift files in `RemTUIKit/` or `RemTUI/`
   - Edit Rust files in `rust-core/`
   - Update UniFFI interface in `rem_core.udl` if needed

3. **Testing**
   ```bash
   # Quick tests during development
   make quick-test
   
   # Full test suite before committing
   make test
   ```

4. **Quality Checks**
   ```bash
   # Format code
   make fmt
   
   # Run linters
   make lint
   
   # Full quality check
   make check
   ```

### Adding New Features

#### Adding Rust TUI Features

1. **Update Interface** (`rem_core.udl`)
   ```idl
   namespace rem_core {
       [Throws=RemError]
       sequence<TUIAction> new_feature_function(NewDataType data);
   };
   
   dictionary NewDataType {
       string field1;
       u32 field2;
   };
   ```

2. **Implement in Rust** (`lib.rs`)
   ```rust
   #[uniffi::export]
   pub fn new_feature_function(data: NewDataType) -> Result<Vec<TUIAction>, RemError> {
       // Implementation
   }
   ```

3. **Regenerate Bindings**
   ```bash
   make build-uniffi
   ```

4. **Use from Swift** (`main.swift`)
   ```swift
   let actions = try newFeatureFunction(data: data)
   ```

#### Adding Swift System Features

1. **Implement in Swift** (`RemTUIKit/`)
   ```swift
   public class NewService {
       // Implementation
   }
   ```

2. **Add to Package** (`Package.swift`)
   ```swift
   // Add to targets if needed
   ```

3. **Test**
   ```bash
   make test-swift
   ```

4. **Integrate** (`main.swift`)
   ```swift
   let service = NewService()
   // Use service
   ```

### UniFFI Integration Workflow

1. **Design Interface**
   - Define data types in `rem_core.udl`
   - Specify function signatures
   - Handle error cases

2. **Implement Rust Side**
   - Add `#[uniffi::export]` to functions
   - Implement error handling
   - Add tests

3. **Generate Bindings**
   ```bash
   make build-uniffi
   ```

4. **Integrate Swift Side**
   - Use generated Swift functions
   - Handle errors appropriately
   - Add Swift tests

5. **Test Integration**
   ```bash
   make test-integration
   ```

## Testing Strategy

### Test Structure

```
Testing Layers:
├── Unit Tests
│   ├── Rust Core Tests (cargo test)
│   └── Swift Package Tests (swift test)
├── Integration Tests  
│   ├── UniFFI Interface Tests
│   └── Swift-Rust Communication Tests
├── System Tests
│   ├── Permission Flow Tests
│   └── EventKit Integration Tests
└── Migration Validation
    ├── Architecture Tests
    └── Performance Benchmarks
```

### Running Tests

```bash
# All tests
make test

# Individual test suites
make test-rust           # Rust unit tests
make test-swift          # Swift unit tests  
make test-integration    # Integration tests
make test-migration      # Architecture validation

# Specific test patterns
cd rust-core && cargo test tui_
cd RemTUIKit && swift test --filter PermissionTests
```

### Test Development

#### Rust Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_initialization() {
        let lists = vec![/* test data */];
        let mut app = TUIApp::new(lists).unwrap();
        // Test assertions
    }
}
```

#### Swift Tests
```swift
import XCTest
@testable import RemTUIKit

final class RemindersServiceTests: XCTestCase {
    func testPermissionFlow() async throws {
        let service = RemindersService()
        // Test async permission flow
    }
}
```

## Debugging Guide

### Debugging Rust Components

```bash
# Run with debug logging
make debug-rust

# Rust-specific debugging
cd rust-core
RUST_LOG=debug cargo run

# Use debugger
cd rust-core
cargo build
lldb target/debug/rem-core
```

#### Rust Debugging Tips

1. **Add Debug Logging**
   ```rust
   use tracing::{debug, info, warn, error};
   
   debug!("Processing action: {:?}", action);
   ```

2. **Use Debug Assertions**
   ```rust
   debug_assert!(!lists.is_empty(), "Lists should not be empty");
   ```

3. **Print Debugging**
   ```rust
   eprintln!("Debug: TUI state = {:?}", self.current_view);
   ```

### Debugging Swift Components

```bash
# Run Swift in debug mode
make debug-swift

# Swift-specific debugging
cd RemTUI
swift run --configuration debug RemTUI

# Use Xcode debugger
cd RemTUI
swift package generate-xcodeproj
open RemTUI.xcodeproj
```

#### Swift Debugging Tips

1. **Add Logging**
   ```swift
   print("🔍 Debug: Loading \(lists.count) lists")
   ```

2. **Error Context**
   ```swift
   do {
       let result = try await operation()
   } catch {
       print("❌ Error in \(#function): \(error)")
       throw error
   }
   ```

### Debugging UniFFI Integration

```bash
# Debug the binding generation
make debug-build

# Check generated files
ls -la RemTUIKit/Sources/RemTUIKit/rem_core*

# Verify library linking
otool -L RemTUIKit/Sources/RemTUIKit/librem_core.dylib
```

#### UniFFI Debugging Tips

1. **Check Interface Matches**
   - Ensure `rem_core.udl` types match Rust exactly
   - Verify function signatures are correct

2. **Debug Data Transfer**
   ```rust
   #[uniffi::export]
   pub fn debug_data_transfer(data: ReminderList) -> ReminderList {
       eprintln!("Received: {:?}", data);
       data // Echo back for verification
   }
   ```

3. **Error Handling**
   ```swift
   do {
       let result = try startTui(lists: lists)
   } catch RemError.TuiError(let message) {
       print("TUI Error: \(message)")
   } catch {
       print("Unexpected error: \(error)")
   }
   ```

### Permission Debugging

```bash
# Check current permissions
make check-permissions

# Debug permission flow
make debug-permissions

# Reset permissions (requires admin)
sudo tccutil reset Reminders
```

## Performance Optimization

### Profiling

```bash
# Run benchmarks
make benchmark

# Profile with Instruments
make profile
# Then use Xcode Instruments

# Memory profiling
cd RemTUI
/usr/bin/time -l .build/release/RemTUI
```

### Optimization Areas

#### Swift Performance

1. **Async Concurrency**
   ```swift
   // Use TaskGroup for concurrent operations
   return try await withThrowingTaskGroup(of: ReminderList.self) { group in
       for calendar in calendars {
           group.addTask {
               return try await self.processCalendar(calendar)
           }
       }
       // Collect results
   }
   ```

2. **Memory Management**
   ```swift
   // Use weak references to avoid cycles
   weak var weakSelf = self
   
   // Minimize object allocations in hot paths
   ```

#### Rust Performance

1. **Minimize Allocations**
   ```rust
   // Use string slices instead of String where possible
   fn process_title(title: &str) -> &str {
       title.trim()
   }
   
   // Reuse Vec buffers
   vec.clear();
   vec.extend_from_slice(&new_data);
   ```

2. **Efficient Rendering**
   ```rust
   // Only re-render when state changes
   if self.needs_redraw {
       terminal.draw(|f| self.ui(f))?;
       self.needs_redraw = false;
   }
   ```

#### UniFFI Performance

1. **Minimize Boundary Crossings**
   ```rust
   // Batch operations instead of individual calls
   #[uniffi::export]
   pub fn process_batch(actions: Vec<TUIAction>) -> Result<BatchResult, RemError>
   ```

2. **Zero-Copy Where Possible**
   ```rust
   // Use borrowed types for read-only operations
   pub fn analyze_data(data: &[ReminderList]) -> Statistics
   ```

## Contributing Guidelines

### Code Style

#### Rust
- Use `cargo fmt` for formatting
- Pass `cargo clippy --all-targets --all-features -- -D warnings`
- Add documentation for public APIs
- Use meaningful variable names
- Prefer explicit error handling over `unwrap()`

#### Swift
- Follow [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)
- Use `swiftformat` if available
- Pass `swiftlint` if available
- Use meaningful names and clear structure
- Handle errors gracefully with contextual messages

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/description
   ```

2. **Development**
   - Make changes with tests
   - Run `make check` and `make test`
   - Update documentation if needed

3. **Commit**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

4. **Pre-submission**
   ```bash
   make ci  # Run CI pipeline locally
   ```

5. **Submit PR**
   - Clear description of changes
   - Reference any issues
   - Include test results

### Code Review Checklist

- [ ] Code follows style guidelines
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] Performance implications considered
- [ ] Error handling implemented
- [ ] UniFFI interface updated if needed
- [ ] Migration impact assessed

## Troubleshooting

### Common Issues

#### Build Failures

**Rust Compilation Errors**
```bash
# Clean and rebuild
make clean-all
make build-rust

# Check dependencies
cd rust-core && cargo check
```

**Swift Compilation Errors**
```bash
# Clean Swift build
make clean
cd RemTUIKit && swift package clean
cd RemTUI && swift package clean

# Rebuild
make build-swift
```

**UniFFI Binding Issues**
```bash
# Regenerate bindings
make clean
make build-uniffi

# Check UDL syntax
cd rust-core && cargo check
```

#### Runtime Issues

**Permission Denied**
```bash
# Check permission status
make check-permissions

# Reset and retry
sudo tccutil reset Reminders
make run
```

**Library Loading Issues**
```bash
# Check library path
ls -la RemTUIKit/Sources/RemTUIKit/librem_core.dylib

# Verify linking
otool -L RemTUIKit/Sources/RemTUIKit/librem_core.dylib
```

**Performance Issues**
```bash
# Profile the application
make profile

# Check system resources
make check-system

# Run benchmarks
make benchmark
```

### Getting Help

1. **Check Documentation**
   - This developer guide
   - README.md for user information
   - MIGRATION.md for architecture details

2. **Run Diagnostics**
   ```bash
   make check-system    # System requirements
   make show-arch       # Architecture status
   make debug-build     # Build debugging
   ```

3. **Community Support**
   - GitHub Issues for bug reports
   - GitHub Discussions for questions
   - Code review for improvements

---

This developer guide provides comprehensive information for working with the Rem TUI codebase. For additional questions or clarifications, please refer to the other documentation files or create an issue in the repository.