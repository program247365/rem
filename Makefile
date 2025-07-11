# Makefile for rem - Fast TUI for Apple Reminders

.PHONY: build run test fmt lint doc clean help install dev check all

# Default target
all: check build

# Build the project
build:
	cargo build

# Build in release mode
build-release:
	cargo build --release

# Run the application
run:
	cargo run

# Run with custom tick/frame rates
run-dev:
	cargo run -- --tick-rate 2.0 --frame-rate 30.0

# Run tests
test:
	cargo test --locked --all-features --workspace

# Format code
fmt:
	cargo fmt --all

# Check formatting
fmt-check:
	cargo fmt --all --check

# Run clippy linting
lint:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

# Generate documentation
doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples

# Open documentation in browser
doc-open:
	cargo doc --no-deps --document-private-items --all-features --workspace --examples --open

# Clean build artifacts
clean:
	cargo clean

# Install the binary
install:
	cargo install --path .

# Development mode - run with auto-reload (requires cargo-watch)
dev:
	cargo watch -x run

# Check everything (format, lint, test)
check: fmt-check lint test

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  build-release- Build in release mode"
	@echo "  run          - Run the application"
	@echo "  run-dev      - Run with custom tick/frame rates"
	@echo "  test         - Run tests"
	@echo "  fmt          - Format code"
	@echo "  fmt-check    - Check code formatting"
	@echo "  lint         - Run clippy linting"
	@echo "  doc          - Generate documentation"
	@echo "  doc-open     - Generate and open documentation"
	@echo "  clean        - Clean build artifacts"
	@echo "  install      - Install the binary"
	@echo "  dev          - Run in development mode (requires cargo-watch)"
	@echo "  check        - Run all checks (format, lint, test)"
	@echo "  all          - Run check and build"
	@echo "  help         - Show this help message"