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

func createTestReminder() async {
    let granted = await requestPermission()
    guard granted else {
        print("❌ Permission denied")
        return
    }
    
    let calendars = eventStore.calendars(for: .reminder)
    guard let testList = calendars.first(where: { $0.title == "TEST_LIST" }) else {
        print("❌ TEST_LIST not found")
        return
    }
    
    let reminder = EKReminder(eventStore: eventStore)
    reminder.title = "Test Delete Reminder"
    reminder.calendar = testList
    reminder.isCompleted = false
    
    do {
        try eventStore.save(reminder, commit: true)
        print("✅ Created test reminder: '\(reminder.title!)' with ID: \(reminder.calendarItemIdentifier)")
    } catch {
        print("❌ Failed to create reminder: \(error)")
    }
}

Task {
    await createTestReminder()
    exit(0)
}

RunLoop.main.run()