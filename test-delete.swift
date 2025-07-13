#!/usr/bin/env swift

import Foundation
import EventKit

// Simple test script to verify delete functionality works
// Run this to test EventKit deletion without the TUI

let eventStore = EKEventStore()

func requestPermission() async -> Bool {
    return await withCheckedContinuation { continuation in
        eventStore.requestFullAccessToReminders { granted, error in
            continuation.resume(returning: granted)
        }
    }
}

func testDeleteReminder() async {
    print("🧪 Testing reminder deletion functionality...")
    
    // Request permission
    let granted = await requestPermission()
    guard granted else {
        print("❌ Permission denied")
        return
    }
    
    print("✅ Permission granted")
    
    // Get calendars
    let calendars = eventStore.calendars(for: .reminder)
    print("📋 Found \(calendars.count) reminder calendars")
    
    // Find a test reminder
    for calendar in calendars {
        print("🔍 Checking calendar: \(calendar.title)")
        
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        let reminders = try! await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders ?? [])
            }
        }
        
        print("📝 Found \(reminders.count) reminders in \(calendar.title)")
        
        // Find a reminder with "test" in the title
        if let testReminder = reminders.first(where: { $0.title?.lowercased().contains("test") == true }) {
            print("🎯 Found test reminder: '\(testReminder.title ?? "")' (ID: \(testReminder.calendarItemIdentifier))")
            
            // Test deletion
            do {
                try eventStore.remove(testReminder, commit: true)
                print("✅ Successfully deleted test reminder!")
                return
            } catch {
                print("❌ Failed to delete reminder: \(error)")
                return
            }
        }
    }
    
    print("⚠️ No test reminder found. Create a reminder with 'test' in the title to test deletion.")
}

// Run the test
Task {
    await testDeleteReminder()
    exit(0)
}

RunLoop.main.run()