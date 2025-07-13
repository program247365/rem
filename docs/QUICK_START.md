# Quick Start Guide - Rem TUI

Get up and running with Rem TUI in minutes!

## Prerequisites

- **macOS 14.0+** (required for EventKit full access)
- **Terminal app** (Terminal.app, iTerm2, etc.)
- **5 minutes** for setup

## Installation

### Option 1: Quick Install (Recommended)

```bash
# Clone and build
git clone https://github.com/yourusername/rem.git
cd rem
make setup && make build

# Run immediately
make run
```

### Option 2: Manual Install

```bash
# 1. Install dependencies
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Xcode Command Line Tools  
xcode-select --install

# 2. Clone and build
git clone https://github.com/yourusername/rem.git
cd rem
make build

# 3. Run
make run
```

## First Run

1. **Start the app**
   ```bash
   make run
   ```

2. **Grant permissions**
   - Native macOS dialog will appear
   - Click "OK" to grant Reminders access
   - No manual setup required!

3. **Use the interface**
   ```
   ↑↓ or j/k  - Navigate lists
   Enter       - Open selected list
   Space       - Toggle reminder completion
   q or Esc    - Back/Quit
   ```

## Example Session

```bash
# Start the app
$ make run
🍎 Rem - Fast TUI for Apple Reminders
✅ Reminders access already granted
📱 Loading your reminder lists...
✅ Found 3 reminder lists
🚀 Starting TUI interface...

# You'll see your lists:
┌─────────────────── 📝 Your Reminder Lists ───────────────────┐
│                                                              │
│  ▶ ●  Today                                                  │
│      15 reminders                                            │
│                                                              │
│    ●  Shopping                                               │
│      8 reminders                                             │
│                                                              │
│    ●  Work                                                   │
│      23 reminders                                            │
│                                                              │
└──────────────────────────────────────────────────────────────┘

# Press Enter to view reminders:
┌─────────────────── 📝 Reminders ───────────────────────────────┐
│                                                              │
│  ▶ ☐  Call dentist                                          │
│                                                              │
│    ☑  Buy groceries                                         │
│                                                              │
│    ☐  Finish project report                                 │
│      Due: Tomorrow                                           │
│                                                              │
└──────────────────────────────────────────────────────────────┘

# Press Space to toggle completion, q to go back
```

## Keyboard Shortcuts

### Lists View
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `Enter` | Open selected list |
| `q` | Quit |

### Reminders View
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down |
| `k` or `↑` | Move up |
| `Space` or `Enter` | Toggle completion |
| `q` or `Esc` | Back to lists |

## Development Mode

Want to modify the app? Start development mode:

```bash
# Auto-rebuild on file changes
make dev

# Edit code in your preferred editor
# Changes trigger automatic rebuild and restart
```

## Troubleshooting

### Permission Issues

**Problem**: "Permission denied" error
```bash
# Check current permissions
make check-permissions

# Reset permissions if needed (requires admin)
sudo tccutil reset Reminders

# Try again
make run
```

### Build Issues

**Problem**: Build fails
```bash
# Clean and rebuild
make clean-all
make setup
make build
```

**Problem**: Swift/Rust not found
```bash
# Check system requirements
make check-system

# Install missing components
# For Rust: https://rustup.rs/
# For Swift: xcode-select --install
```

### Performance Issues

**Problem**: Slow startup or operation
```bash
# Run benchmarks to identify issues
make benchmark

# Check system resources
make check-system

# Debug with logging
make debug
```

## Advanced Usage

### Debug Mode
```bash
# Run with detailed logging
make debug

# Debug specific components
make debug-rust    # Rust TUI debugging
make debug-swift   # Swift integration debugging
```

### Performance Monitoring
```bash
# Benchmark against previous version
make benchmark

# Profile the application
make profile
```

### Custom Development
```bash
# Build specific components
make build-rust      # Just the Rust core
make build-swift     # Just the Swift wrapper
make build-uniffi   # Regenerate bindings

# Test specific parts
make test-rust       # Rust unit tests
make test-swift      # Swift tests
make test-integration # End-to-end tests
```

## What's Next?

1. **Explore features**: Try navigating different lists, toggling reminders
2. **Customize**: The app respects your Reminders app organization
3. **Contribute**: Check out [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) for contributing
4. **Report issues**: Use GitHub Issues for bugs or feature requests

## Performance Expectations

With the new architecture, you should experience:

- **Startup**: <1 second (vs 18+ seconds with old AppleScript)
- **List loading**: <500ms for typical data
- **Reminder operations**: <200ms per action
- **Memory usage**: 2-5MB typical

## Architecture Benefits

This version uses a **Swift wrapper + Rust core** design:

- **Swift**: Native macOS integration, automatic permissions
- **Rust**: High-performance TUI, memory safety
- **UniFFI**: Type-safe communication between layers

## Get Help

- **Documentation**: Check [docs/](docs/) folder
- **Issues**: GitHub Issues for bug reports
- **Discussions**: GitHub Discussions for questions
- **Contributing**: [DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)

---

Enjoy using Rem TUI! 🎉