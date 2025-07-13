#!/usr/bin/env swift

import Foundation
import EventKit

let eventStore = EKEventStore()

func requestPermission() async -> Bool {
    return await withCheckedContinuation { continuation in
        eventStore.requestFullAccessToReminders { granted, error in
            continuation.resume(returning: granted)
        }
    }
}

func setupTestReminder() async {
    print("🎯 Setting up test reminder for TUI testing...")
    
    let granted = await requestPermission()
    guard granted else {
        print("❌ Permission denied")
        return
    }
    
    // Find TEST_LIST
    let calendars = eventStore.calendars(for: .reminder)
    guard let testList = calendars.first(where: { $0.title == "TEST_LIST" }) else {
        print("❌ TEST_LIST not found")
        return
    }
    
    let reminder = EKReminder(eventStore: eventStore)
    reminder.title = "Test TUI Creation"
    reminder.notes = "This will be replaced by TUI-created reminder"
    reminder.calendar = testList
    reminder.isCompleted = false
    reminder.priority = 1
    
    do {
        try eventStore.save(reminder, commit: true)
        print("✅ Created test reminder: '\(reminder.title!)' with ID: \(reminder.calendarItemIdentifier)")
        print("📋 TEST_LIST is ready for TUI testing!")
    } catch {
        print("❌ Failed to create test reminder: \(error)")
    }
}

Task {
    await setupTestReminder()
    exit(0)
}

RunLoop.main.run()