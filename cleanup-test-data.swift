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

func cleanupTestData() async {
    print("🧹 Cleaning up test data...")
    
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
    
    let predicate = eventStore.predicateForReminders(in: [testList])
    
    let reminders = try! await withCheckedThrowingContinuation { continuation in
        eventStore.fetchReminders(matching: predicate) { reminders in
            continuation.resume(returning: reminders ?? [])
        }
    }
    
    print("📝 Found \(reminders.count) reminders to cleanup")
    
    for reminder in reminders {
        do {
            try eventStore.remove(reminder, commit: true)
            print("🗑️ Deleted: '\(reminder.title ?? "")'")
        } catch {
            print("❌ Failed to delete '\(reminder.title ?? "")': \(error)")
        }
    }
    
    print("✅ Cleanup completed!")
}

Task {
    await cleanupTestData()
    exit(0)
}

RunLoop.main.run()