#!/bin/bash

set -e

echo "🦀 Building Rust core library..."
cd rust-core

# Build the Rust library
cargo build --release

echo "🔗 Generating UniFFI bindings..."
# Generate Swift bindings from the UDL file
cargo run --bin uniffi-bindgen generate src/rem_core.udl --language swift --out-dir ../RemTUIKit/Sources/RemTUIKit/

# Copy the generated library to the Swift package
echo "📦 Copying Rust library..."
cp target/release/librem_core.dylib ../RemTUIKit/Sources/RemTUIKit/

cd ..

echo "🍎 Building Swift package..."
cd RemTUIKit
swift build -c release
cd ..

echo "🎯 Building final executable..."
cd RemTUI
swift build -c release
cd ..

echo "✅ Build complete!"
echo ""
echo "📍 Generated files:"
echo "   • rust-core/target/release/librem_core.dylib - Rust core library"
echo "   • RemTUIKit/Sources/RemTUIKit/*.swift - Generated UniFFI bindings"
echo "   • RemTUI/.build/release/RemTUI - Final executable"
echo ""
echo "🚀 To run: cd RemTUI && .build/release/RemTUI"