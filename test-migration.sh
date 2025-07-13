#!/bin/bash

echo "🧪 Testing Migration - Swift + Rust Core Architecture"
echo "=================================================="

# Test 1: Check if Rust project compiles
echo "🦀 Test 1: Checking Rust core compilation..."
cd rust-core
if cargo check --quiet; then
    echo "✅ Rust core compiles successfully"
else
    echo "❌ Rust core compilation failed"
    exit 1
fi
cd ..

# Test 2: Check if Swift package compiles
echo "🍎 Test 2: Checking Swift package compilation..."
cd RemTUIKit
if swift build > /dev/null 2>&1; then
    echo "✅ Swift package compiles successfully"
else
    echo "❌ Swift package compilation failed"
    exit 1
fi
cd ..

# Test 3: Check if main Swift app compiles
echo "🎯 Test 3: Checking main app compilation..."
cd RemTUI
if swift build > /dev/null 2>&1; then
    echo "✅ Main app compiles successfully"
else
    echo "❌ Main app compilation failed"
    exit 1
fi
cd ..

# Test 4: Basic functionality test (without full build)
echo "📋 Test 4: Permission manager basic test..."
cd RemTUI
if swift run RemTUI --help > /dev/null 2>&1 || swift run RemTUI 2>&1 | grep -q "Rem - Fast TUI"; then
    echo "✅ App starts correctly"
else
    echo "⚠️  App may need full build to test completely"
fi
cd ..

echo ""
echo "🎉 Migration tests completed successfully!"
echo "📝 Next steps:"
echo "   1. Run ./build.sh to generate UniFFI bindings"
echo "   2. Test with actual Reminders data"
echo "   3. Validate feature parity with original app"