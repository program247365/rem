# Rem - Apple Reminders TUI

A fast, beautiful Terminal User Interface (TUI) for Apple Reminders built with a modern Swift wrapper + Rust core architecture. Rem provides a keyboard-driven interface to view, navigate, and manage your Apple Reminders directly from the terminal with native macOS integration.

[![CI](https://github.com//rem/workflows/CI/badge.svg)](https://github.com//rem/actions)

## ✨ Features

- 🚀 **Lightning Fast**: Native EventKit integration eliminates AppleScript delays (18+ seconds → <1 second)
- 📱 **Real Apple Reminders Data**: Direct integration with macOS Reminders app
- ⌨️ **Vim-style Navigation**: Intuitive keyboard shortcuts (j/k, arrow keys)
- 🎨 **Beautiful UI**: Modern terminal interface with colors, emojis, and rounded borders
- ✅ **Full Management**: Create, toggle, delete reminders with comprehensive form interface
- 🔍 **Live Data**: Real-time access to your actual reminders and lists
- 🛡️ **Native Permissions**: Automatic permission handling with native macOS dialogs
- 🏗️ **Modern Architecture**: Type-safe Swift-Rust communication via UniFFI
- 📝 **Rich Creation**: Create reminders with title, notes, due dates, lists, and priorities

## 🖼️ Screenshots

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
┌──────────────────────── Controls ────────────────────────┐
│  ↑↓ or j/k navigate  ⏎ select  c create  q quit         │
└────────────────────────────────────────────────────────────┘
```

## 🏗️ Architecture Overview

Rem uses a modern **Swift wrapper + Rust core** architecture that separates system integration from TUI logic:

```
┌─────────────────────────────────────────────────────────┐
│                    Swift Wrapper                       │
│  ┌─────────────────┐  ┌─────────────────────────────┐   │
│  │ RemindersService│  │   PermissionManager         │   │
│  │  (EventKit)     │  │  (Native Dialogs)           │   │
│  └─────────────────┘  └─────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                   UniFFI Bridge                         │
│             (Type-Safe Rust ↔ Swift)                    │
├─────────────────────────────────────────────────────────┤
│                    Rust Core                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              Ratatui TUI Engine                     │ │
│  │   ┌───────────┐ ┌──────────┐ ┌─────────────────┐    │ │
│  │   │   Lists   │ │Reminders │ │   Navigation    │    │ │
│  │   │Component  │ │Component │ │    & Events     │    │ │
│  │   └───────────┘ └──────────┘ └─────────────────┘    │ │
│  └─────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│                 Apple Reminders                         │
│                   (macOS App)                           │
└─────────────────────────────────────────────────────────┘
```

### 🔄 Data Flow

1. **Swift Entry Point**: Handles permissions and loads data via native EventKit
2. **Native Integration**: Fast, reliable access to Reminders data 
3. **UniFFI Bridge**: Type-safe data conversion between Swift and Rust
4. **Rust TUI**: High-performance terminal interface and user interaction
5. **Action Processing**: Swift receives actions and updates data accordingly

## 📁 Project Structure

```
RemTUI/                           # New Architecture
├── RemTUIKit/                    # Swift Package (System Integration)
│   ├── Package.swift
│   ├── Sources/RemTUIKit/
│   │   ├── RemindersService.swift    # Native EventKit wrapper
│   │   ├── PermissionManager.swift   # Native permission handling
│   │   └── [Generated UniFFI files]  # Rust bindings
│   └── Tests/RemTUIKitTests/
├── rust-core/                   # Rust Library (TUI Core)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs              # UniFFI exports
│   │   ├── rem_core.udl        # Interface definition
│   │   └── tui/               # Ratatui implementation
│   │       ├── app.rs         # Main TUI application
│   │       ├── components.rs  # UI components
│   │       └── events.rs      # Event handling
├── RemTUI/                     # Swift Executable
│   ├── Package.swift
│   ├── Sources/main.swift      # Application entry point
│   └── Info.plist             # Permissions & metadata
├── build.sh                    # Build coordination script
├── test-migration.sh           # Validation script
├── Makefile                    # Development workflow
├── MIGRATION.md               # Architecture migration guide
├── DEVELOPMENT.md             # Feature development guide
└── ARCHITECTURE.md            # Detailed architecture documentation
```

## 🚀 Installation

### Prerequisites

- **macOS 14.0+** (required for full EventKit access)
- **Rust 1.70+** with Cargo
- **Swift 5.9+** with Package Manager
- **Xcode Command Line Tools**

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/rem.git
cd rem

# Build everything with UniFFI bindings
make build

# Run the application
make run
```

### Development Installation

```bash
# Install with development tools
make install-dev

# Run tests
make test

# Run with debug logging
make debug
```

## 🎮 Usage

### Basic Commands

```bash
# Start the application (full build + run)
make run

# Run without rebuilding (preserves UniFFI checksums - recommended)
make run-direct

# Run directly:
cd RemTUI && .build/release/RemTUI

# Run with debug output
make debug

# Run development version
make dev
```

> **Note**: Use `make run-direct` to avoid UniFFI checksum issues when the TUI integration is already built. This builds only the Swift executable without regenerating Rust bindings.

### Navigation & Controls

**Lists View:**
- `j`/`k` or `↑`/`↓` - Navigate between lists
- `Enter` - Open selected list
- `c` - Create new reminder
- `q` - Quit application

**Reminders View:**
- `j`/`k` or `↑`/`↓` - Navigate between reminders
- `Space` or `Enter` - Toggle reminder completion
- `dd` or `Delete` - Delete selected reminder (vim-style)
- `c` - Create new reminder
- `q` or `Esc` - Go back to lists

**Create Reminder Form:**
- `Tab` - Navigate between form fields
- `↑`/`↓` - Change list/priority selections
- `Ctrl+S` - Save and create reminder
- `q` or `Esc` - Cancel and return

**Form Fields:**
- **Title** - Text input for reminder title (required)
- **Notes** - Multi-line text input for notes
- **Date** - Due date in ISO8601 format (e.g., 2024-12-31T23:59:59Z)
- **List** - Select target reminder list
- **Priority** - Set priority level (0-9, where 0 = none)

### Permissions

On first run, Rem will automatically request permission to access your Reminders using native macOS dialogs. No manual setup required!

## 🛠️ Development

### Development Workflow

```bash
# Setup development environment
make setup

# Build components individually
make build-rust      # Build Rust core only
make build-uniffi    # Build Rust + generate UniFFI bindings
make build-swift     # Build Swift package only
make build          # Build everything

# Running
make run            # Full build + run
make run-direct     # Run without rebuilding (preserves checksums)
make run-debug      # Run in debug mode

# Development
make dev-swift      # Watch Swift files only (preserves checksums)
make fix-checksums  # Fix UniFFI checksums after rebuild

# Development and testing
make dev            # Run in development mode
make test           # Run all tests
make test-rust      # Test Rust core only
make test-swift     # Test Swift package only

# Code quality
make fmt            # Format all code
make lint           # Run linters
make check          # Check compilation without building

# Debugging
make debug          # Run with debug logging
make debug-rust     # Debug Rust components
make debug-swift    # Debug Swift components

# Cleanup
make clean          # Clean build artifacts
make clean-all      # Clean everything including dependencies
```

### Architecture Details

#### Swift Layer (System Integration)

**RemindersService.swift**
- Native EventKit integration for fast data access
- Concurrent list and reminder loading
- Real-time permission status monitoring
- Error handling with detailed context

**PermissionManager.swift**
- Native permission request dialogs
- Status checking and guidance
- Automatic permission flow handling

#### Rust Layer (TUI Core)

**lib.rs** - UniFFI exports and main interface
**tui/app.rs** - Core TUI application logic with Ratatui
**rem_core.udl** - UniFFI interface definition for type safety

#### Build System

**build.sh** - Coordinates Rust compilation and UniFFI binding generation
**Makefile** - Complete development workflow automation
**test-migration.sh** - Validates architecture and compilation

### Performance Improvements

| Metric | Before (AppleScript) | After (Native EventKit) |
|--------|---------------------|-------------------------|
| Cold Start | 18+ seconds | <1 second |
| List Loading | 2-3 seconds | <500ms |
| Reminder Fetch | 1-2 seconds/list | <200ms/list |
| Permission Setup | Manual guidance | Automatic native dialogs |

## 🧪 Testing

```bash
# Run all tests
make test

# Test individual components
make test-rust          # Rust core tests
make test-swift         # Swift package tests
make test-integration   # Integration tests

# Validate migration
make test-migration     # Architecture validation

# Performance benchmarks
make benchmark          # Performance comparison tests
```

## 🔧 Troubleshooting

### UniFFI Checksum Issues

The most common issue is "UniFFI API checksum mismatch" when rebuilding:

```bash
# Use run-direct to avoid regenerating bindings
make run-direct

# If checksums are out of sync, clean rebuild:
make clean-all
make build
# Then manually update checksums in RemCore.swift if needed
```

**Why this happens**: UniFFI generates checksums to ensure the Rust library matches the Swift bindings. When rebuilding, the checksums may change, requiring manual updates in the generated Swift file.

### Permission Issues

The new architecture handles permissions automatically, but if you encounter issues:

```bash
# Check permission status
make check-permissions

# Reset permissions (if needed)
tccutil reset Reminders

# Debug permission flow
make debug-permissions
```

### Build Issues

```bash
# Clean and rebuild everything
make clean-all
make build

# Check individual components
make check-rust         # Rust compilation check
make check-swift        # Swift compilation check

# Debug build process
make debug-build
```

### Performance Issues

```bash
# Profile the application
make profile

# Debug with detailed logging
make debug

# Check system requirements
make check-system
```

## 🚀 Future Enhancements

The new architecture enables exciting possibilities:

- **📅 Calendar Integration**: Events alongside reminders
- **🔄 Real-time Sync**: Live updates with iCloud synchronization
- **🏷️ Advanced Filtering**: Tag-based and smart list filtering
- **📝 Rich Editing**: In-place reminder editing and creation
- **🎯 Bulk Operations**: Multi-select actions and batch operations
- **📊 Analytics**: Productivity insights and completion tracking

## 🤝 Contributing

We welcome contributions! The new architecture makes it easy to contribute to specific layers:

### Contributing Areas

- **Swift Layer**: EventKit integration, native macOS features, permissions
- **Rust Core**: TUI components, user experience, performance optimizations
- **Documentation**: Guides, examples, architecture explanations

### Getting Started

```bash
# Fork and clone
git clone https://github.com/yourusername/rem.git
cd rem

# Setup development environment
make setup

# Run tests to ensure everything works
make test

# Create feature branch
git checkout -b feature/amazing-feature

# Make changes and test
make dev
make test

# Submit pull request
```

### Adding New Features

See [DEVELOPMENT.md](DEVELOPMENT.md) for a comprehensive guide on adding functionality through all layers of the architecture, including:

- Step-by-step feature implementation
- UniFFI integration patterns
- Key handling and TUI updates
- Swift backend services
- Testing strategies

The development guide uses the delete functionality (`dd` command) as a complete example of implementing features from Rust TUI to Swift EventKit integration.

### Code Style

- **Swift**: Follow Swift API Design Guidelines
- **Rust**: Use `cargo fmt` and pass `cargo clippy`
- **Documentation**: Update relevant docs in `docs/`
- **Tests**: Add tests for new functionality

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Ratatui](https://github.com/ratatui-org/ratatui)** - Excellent TUI framework for Rust
- **[UniFFI](https://github.com/mozilla/uniffi-rs)** - Type-safe FFI between Rust and Swift  
- **Apple EventKit** - Native Reminders integration
- **Rust Community** - Amazing ecosystem and documentation
- **Swift Package Manager** - Modern dependency management

---

**Migration Note**: This version represents a complete architectural upgrade from the previous Rust-only implementation. See [MIGRATION.md](MIGRATION.md) for detailed migration information and performance comparisons.