# Makefile for Rem - Apple Reminders TUI
# Swift Wrapper + Rust Core Architecture

.PHONY: help setup build build-rust build-swift build-uniffi run dev debug test test-rust test-swift test-integration test-migration fmt lint check clean clean-all install-dev check-permissions check-system benchmark profile docs

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
BLUE := \033[0;34m
NC := \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

# Help - show available targets
help: ## Show this help message
	@echo "$(BLUE)Rem - Apple Reminders TUI$(NC)"
	@echo "$(BLUE)Swift Wrapper + Rust Core Architecture$(NC)"
	@echo ""
	@echo "$(GREEN)Setup & Installation:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Setup.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Building:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Build.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Running & Development:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Run.*|.*Dev.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Testing & Quality:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Test.*|.*Check.*|.*Format.*|.*Lint.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Debugging & Troubleshooting:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Debug.*|.*Profile.*|.*Benchmark.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Cleanup:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*Clean.*/ {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Setup and Installation
setup: ## Setup development environment and dependencies
	@echo "$(GREEN)Setting up development environment...$(NC)"
	@echo "$(YELLOW)Checking Rust installation...$(NC)"
	@rustc --version || (echo "$(RED)Rust not found. Install from https://rustup.rs/$(NC)" && exit 1)
	@echo "$(YELLOW)Checking Swift installation...$(NC)"
	@swift --version || (echo "$(RED)Swift not found. Install Xcode Command Line Tools$(NC)" && exit 1)
	@echo "$(YELLOW)Checking UniFFI bindgen...$(NC)"
	@cargo install uniffi-bindgen || echo "$(YELLOW)Installing uniffi-bindgen...$(NC)"
	@echo "$(YELLOW)Installing development tools...$(NC)"
	@cargo install cargo-watch cargo-nextest || echo "$(YELLOW)Optional tools installation complete$(NC)"
	@echo "$(GREEN)✅ Development environment setup complete!$(NC)"

install-dev: setup ## Setup development environment with all tools
	@echo "$(GREEN)Installing additional development tools...$(NC)"
	@cargo install cargo-expand cargo-audit cargo-deny
	@echo "$(GREEN)✅ Development tools installed!$(NC)"

check-system: ## Check system requirements and dependencies
	@echo "$(GREEN)Checking system requirements...$(NC)"
	@echo "$(YELLOW)macOS Version:$(NC)"
	@sw_vers
	@echo "$(YELLOW)Rust Version:$(NC)"
	@rustc --version
	@echo "$(YELLOW)Swift Version:$(NC)"
	@swift --version
	@echo "$(YELLOW)Available disk space:$(NC)"
	@df -h . | tail -1
	@echo "$(GREEN)✅ System check complete!$(NC)"

# Building
build: build-uniffi build-rust build-swift ## Build everything (Rust core + Swift wrapper + UniFFI bindings)
	@echo "$(GREEN)✅ Complete build finished!$(NC)"

build-rust: ## Build Rust core library
	@echo "$(YELLOW)Building Rust core...$(NC)"
	@cd rust-core && cargo build --release
	@echo "$(GREEN)✅ Rust core built successfully!$(NC)"

build-swift: ## Build Swift package and executable
	@echo "$(YELLOW)Building Swift package...$(NC)"
	@cd RemTUIKit && swift build -c release -Xswiftc -I. -Xlinker -L. -Xlinker -lrem_core
	@echo "$(YELLOW)Building Swift executable...$(NC)"
	@cd RemTUI && swift build -c release
	@echo "$(GREEN)✅ Swift components built successfully!$(NC)"

build-uniffi: build-rust ## Build UniFFI bindings and copy dylib
	@echo "$(YELLOW)Generating UniFFI bindings...$(NC)"
	@cd rust-core && cargo run --release --bin uniffi-bindgen generate src/rem_core.udl --language swift --out-dir ../RemTUIKit/Sources/RemTUIKit/
	@echo "$(YELLOW)Copying Rust library...$(NC)"
	@cp rust-core/target/release/librem_core.dylib RemTUIKit/Sources/RemTUIKit/ 2>/dev/null || \
		echo "$(YELLOW)Note: Rust library will be available after first Rust build$(NC)"
	@echo "$(YELLOW)Fixing UniFFI checksums...$(NC)"
	@./fix-checksums.sh
	@echo "$(GREEN)✅ UniFFI bindings generated!$(NC)"

fix-checksums: ## Fix UniFFI checksums after rebuild
	@echo "$(YELLOW)Fixing UniFFI checksums...$(NC)"
	@./fix-checksums.sh

build-debug: ## Build everything in debug mode
	@echo "$(YELLOW)Building in debug mode...$(NC)"
	@cd rust-core && cargo build
	@cd RemTUIKit && swift build
	@cd RemTUI && swift build
	@echo "$(GREEN)✅ Debug build complete!$(NC)"

# Running and Development
run: build ## Run the application
	@echo "$(GREEN)🚀 Starting Rem TUI...$(NC)"
	@cd RemTUI && .build/release/RemTUI

run-debug: build-debug ## Run the application in debug mode
	@echo "$(GREEN)🚀 Starting Rem TUI (debug)...$(NC)"
	@cd RemTUI && .build/debug/RemTUI

run-direct: ## Run the application without rebuilding (preserves checksums)
	@echo "$(GREEN)🚀 Starting Rem TUI (direct)...$(NC)"
	@echo "$(YELLOW)Building Swift executable only...$(NC)"
	@cd RemTUI && swift build -c release
	@echo "$(YELLOW)Running TUI...$(NC)"
	@cd RemTUI && ./.build/release/RemTUI

dev: ## Run in development mode with auto-rebuild
	@echo "$(GREEN)🔄 Starting development mode...$(NC)"
	@echo "$(YELLOW)Watching for changes... Press Ctrl+C to stop$(NC)"
	@cargo watch -w rust-core -w RemTUIKit -w RemTUI -s "make build-debug && make run-debug"

dev-swift: ## Run development mode with Swift-only rebuilds (preserves checksums)
	@echo "$(GREEN)🔄 Starting Swift-only development mode...$(NC)"
	@echo "$(YELLOW)Watching for Swift changes... Press Ctrl+C to stop$(NC)"
	@echo "$(BLUE)Note: This preserves UniFFI checksums by not rebuilding Rust$(NC)"
	@cargo watch -w RemTUI -w RemTUIKit/Sources/RemTUIKit/PermissionManager.swift -w RemTUIKit/Sources/RemTUIKit/RemindersService.swift -s "make run-direct"

debug: build-debug ## Run with debug logging enabled
	@echo "$(GREEN)🐛 Starting with debug logging...$(NC)"
	@cd RemTUI && DEBUG=1 .build/debug/RemTUI

debug-rust: ## Debug Rust components specifically
	@echo "$(YELLOW)Debugging Rust core...$(NC)"
	@cd rust-core && RUST_LOG=debug cargo run

debug-swift: ## Debug Swift components specifically  
	@echo "$(YELLOW)Debugging Swift components...$(NC)"
	@cd RemTUI && swift run RemTUI

debug-build: ## Debug the build process
	@echo "$(YELLOW)Debugging build process...$(NC)"
	@echo "$(BLUE)Rust build:$(NC)"
	@cd rust-core && cargo build -v
	@echo "$(BLUE)Swift build:$(NC)"
	@cd RemTUIKit && swift build -v
	@cd RemTUI && swift build -v

# Testing and Quality
test: test-rust test-swift ## Run all tests
	@echo "$(GREEN)✅ All tests completed!$(NC)"

test-rust: ## Run Rust core tests
	@echo "$(YELLOW)Running Rust tests...$(NC)"
	@cd rust-core && cargo test --locked --all-features

test-swift: ## Run Swift package tests
	@echo "$(YELLOW)Running Swift tests...$(NC)"
	@cd RemTUIKit && swift test
	@echo "$(YELLOW)Testing Swift executable build...$(NC)"
	@cd RemTUI && swift build


test-permissions: ## Test permissions functionality
	@echo "$(YELLOW)Testing permissions...$(NC)"
	@cd RemTUI && .build/release/RemTUI --help || echo "$(YELLOW)Build required first$(NC)"

check: check-rust check-swift ## Check all code quality (formatting, linting, compilation)

check-rust: ## Check Rust code quality
	@echo "$(YELLOW)Checking Rust code...$(NC)"
	@cd rust-core && cargo fmt --check
	@cd rust-core && cargo clippy --all-targets --all-features -- -D warnings
	@cd rust-core && cargo check

check-swift: ## Check Swift code quality
	@echo "$(YELLOW)Checking Swift code...$(NC)"
	@cd RemTUIKit && swift build
	@cd RemTUI && swift build

fmt: fmt-rust fmt-swift ## Format all code

fmt-rust: ## Format Rust code
	@echo "$(YELLOW)Formatting Rust code...$(NC)"
	@cd rust-core && cargo fmt

fmt-swift: ## Format Swift code
	@echo "$(YELLOW)Formatting Swift code...$(NC)"
	@echo "$(BLUE)Note: Swift formatting requires swiftformat tool$(NC)"
	@which swiftformat >/dev/null && (cd RemTUIKit && swiftformat .) || echo "$(YELLOW)Install swiftformat for Swift formatting$(NC)"

lint: lint-rust lint-swift ## Run all linters

lint-rust: ## Run Rust linter (clippy)
	@echo "$(YELLOW)Running Rust linter...$(NC)"
	@cd rust-core && cargo clippy --all-targets --all-features -- -D warnings

lint-swift: ## Run Swift linter
	@echo "$(YELLOW)Running Swift linter...$(NC)"
	@which swiftlint >/dev/null && (cd RemTUIKit && swiftlint) || echo "$(YELLOW)Install swiftlint for Swift linting$(NC)"

# Debugging and Profiling
benchmark: build ## Run performance benchmarks
	@echo "$(YELLOW)Running performance benchmarks...$(NC)"
	@echo "$(BLUE)Cold start benchmark:$(NC)"
	@time (cd RemTUI && .build/release/RemTUI --help)
	@echo "$(BLUE)Memory usage:$(NC)"
	@cd RemTUI && /usr/bin/time -l .build/release/RemTUI --help 2>&1 | grep "maximum resident set size"

profile: build-debug ## Profile the application
	@echo "$(YELLOW)Profiling application...$(NC)"
	@echo "$(BLUE)Use Instruments.app or other profiling tools with:$(NC)"
	@echo "cd RemTUI && .build/debug/RemTUI"

debug-permissions: ## Debug permission issues
	@echo "$(YELLOW)Debugging permissions...$(NC)"
	@echo "$(BLUE)Current Reminders permissions:$(NC)"
	@tccutil reset Reminders 2>/dev/null || echo "$(YELLOW)Run as admin to reset permissions$(NC)"
	@echo "$(BLUE)Running permission test:$(NC)"
	@cd RemTUI && .build/release/RemTUI 2>&1 | head -20

check-permissions: ## Check current permission status
	@echo "$(YELLOW)Checking Reminders permissions...$(NC)"
	@sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db "SELECT service, client, allowed FROM access WHERE service='kTCCServiceReminders';" 2>/dev/null || echo "$(YELLOW)Unable to read TCC database$(NC)"

# Documentation
docs: ## Generate documentation
	@echo "$(YELLOW)Generating documentation...$(NC)"
	@cd rust-core && cargo doc --no-deps --document-private-items --all-features
	@echo "$(GREEN)✅ Documentation generated!$(NC)"
	@echo "$(BLUE)Open rust-core/target/doc/rem_core/index.html to view$(NC)"

docs-open: docs ## Generate and open documentation
	@open rust-core/target/doc/rem_core/index.html

# Cleanup
clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	@cd rust-core && cargo clean
	@cd RemTUIKit && swift package clean
	@cd RemTUI && swift package clean
	@rm -f RemTUIKit/Sources/RemTUIKit/librem_core.dylib
	@rm -f RemTUIKit/Sources/RemTUIKit/rem_core.swift
	@rm -f RemTUIKit/Sources/RemTUIKit/rem_coreFFI.h
	@echo "$(GREEN)✅ Cleanup complete!$(NC)"

clean-all: clean ## Clean everything including dependencies
	@echo "$(YELLOW)Deep cleaning...$(NC)"
	@cd rust-core && rm -rf target/
	@cd RemTUIKit && rm -rf .build/
	@cd RemTUI && rm -rf .build/
	@echo "$(GREEN)✅ Deep cleanup complete!$(NC)"

# Utility targets
show-arch: ## Show current architecture status
	@echo "$(GREEN)Current Architecture Status:$(NC)"
	@echo "$(BLUE)Project Structure:$(NC)"
	@tree -L 3 -I 'target|.build|.git' . 2>/dev/null || ls -la
	@echo ""
	@echo "$(BLUE)Rust Core:$(NC)"
	@ls -la rust-core/src/ 2>/dev/null || echo "Not found"
	@echo ""
	@echo "$(BLUE)Swift Package:$(NC)"
	@ls -la RemTUIKit/Sources/RemTUIKit/ 2>/dev/null || echo "Not found"
	@echo ""
	@echo "$(BLUE)Generated Bindings:$(NC)"
	@ls -la RemTUIKit/Sources/RemTUIKit/*core* 2>/dev/null || echo "No UniFFI bindings generated yet"

# Quick development workflow shortcuts
quick-build: build-rust build-uniffi ## Quick build (Rust + bindings only)
	@echo "$(GREEN)✅ Quick build complete!$(NC)"

quick-test: test-rust ## Quick test (Rust only)
	@echo "$(GREEN)✅ Quick test complete!$(NC)"

quick-run: quick-build ## Quick build and run
	@echo "$(GREEN)🚀 Quick run...$(NC)"
	@cd RemTUI && swift run RemTUI

# Installation targets
install: build ## Install the built application
	@echo "$(YELLOW)Installing RemTUI...$(NC)"
	@mkdir -p ~/bin
	@cp RemTUI/.build/release/RemTUI ~/bin/rem-tui
	@echo "$(GREEN)✅ Installed to ~/bin/rem-tui$(NC)"
	@echo "$(BLUE)Add ~/bin to your PATH to use 'rem-tui' command$(NC)"

uninstall: ## Uninstall the application
	@echo "$(YELLOW)Uninstalling RemTUI...$(NC)"
	@rm -f ~/bin/rem-tui
	@echo "$(GREEN)✅ Uninstalled!$(NC)"

# Development workflow
fresh-start: clean-all setup build test ## Complete fresh development setup
	@echo "$(GREEN)🎉 Fresh development environment ready!$(NC)"

ci: check test ## Run CI pipeline (checks + tests)
	@echo "$(GREEN)✅ CI pipeline complete!$(NC)"

# CI-specific targets (used by GitHub Actions)
ci-check-rust-formatting: ## CI: Check Rust formatting
	@cd rust-core && cargo fmt --check

ci-run-rust-linter: ## CI: Run Rust linter
	@cd rust-core && cargo clippy --all-targets --all-features -- -D warnings

ci-build-uniffi-bindgen: ## CI: Build UniFFI bindgen
	@cd rust-core && cargo build --release --bin uniffi-bindgen

ci-build-rust-core: ## CI: Build Rust core
	@make build-rust

ci-generate-uniffi-bindings: ## CI: Generate UniFFI bindings
	@make build-uniffi

ci-build-swift-package: ## CI: Build Swift package
	@cd RemTUIKit && swift build -c release

ci-build-swift-executable: ## CI: Build Swift executable
	@cd RemTUI && swift build -c release

ci-build-for-testing: ## CI: Build for testing
	@make build

ci-run-rust-tests: ## CI: Run Rust tests
	@make test-rust

ci-run-swift-tests: ## CI: Run Swift tests
	@make test-swift

ci-check-code-quality: ## CI: Check code quality
	@make check-rust
	@make build-uniffi
	@make check-swift

ci-build-release: ## CI: Build release
	@make build

ci-create-distribution-package: ## CI: Create distribution package
	@mkdir -p dist
	@cp RemTUI/.build/release/RemTUI dist/rem-tui
	@cp rust-core/target/release/librem_core.dylib dist/
	@chmod +x dist/rem-tui

ci-create-tarball: ## CI: Create tarball
	@cd dist && tar -czf rem-tui-macos.tar.gz rem-tui librem_core.dylib