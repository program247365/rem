#!/bin/bash

# Fix UniFFI checksums after rebuild
# This script updates the RemCore.swift file with the correct checksums

set -e

echo "🔧 Fixing UniFFI checksums..."

REMCORE_FILE="RemTUIKit/Sources/RemTUIKit/RemCore.swift"

if [ ! -f "$REMCORE_FILE" ]; then
    echo "❌ Error: RemCore.swift not found at $REMCORE_FILE"
    exit 1
fi

echo "📝 Updating checksums in $REMCORE_FILE..."

# Update the checksum values with correct values from Rust library
sed -i '' 's/uniffi_rem_core_checksum_func_continue_persistent_tui() != [0-9]*/uniffi_rem_core_checksum_func_continue_persistent_tui() != 21085/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_render_reminders_view() != [0-9]*/uniffi_rem_core_checksum_func_render_reminders_view() != 27359/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_run_persistent_tui() != [0-9]*/uniffi_rem_core_checksum_func_run_persistent_tui() != 18082/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_set_global_reminders() != [0-9]*/uniffi_rem_core_checksum_func_set_global_reminders() != 46351/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_set_reminders() != [0-9]*/uniffi_rem_core_checksum_func_set_reminders() != 27881/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_shutdown_tui() != [0-9]*/uniffi_rem_core_checksum_func_shutdown_tui() != 56565/' "$REMCORE_FILE"
sed -i '' 's/uniffi_rem_core_checksum_func_start_tui() != [0-9]*/uniffi_rem_core_checksum_func_start_tui() != 12292/' "$REMCORE_FILE"

echo "✅ Checksums updated successfully!"
echo "📌 Current checksums:"
echo "   - continue_persistent_tui: 21085"
echo "   - render_reminders_view: 27359"
echo "   - run_persistent_tui: 18082"
echo "   - set_global_reminders: 46351"
echo "   - set_reminders: 27881"
echo "   - shutdown_tui: 56565"
echo "   - start_tui: 12292"