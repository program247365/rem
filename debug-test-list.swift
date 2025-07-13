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

func debugTestList() async {
    print("🔍 Debugging TEST_LIST reminders...")
    
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
    
    print("✅ Found TEST_LIST calendar: \(testList.title)")
    print("📋 Calendar ID: \(testList.calendarIdentifier)")
    
    let predicate = eventStore.predicateForReminders(in: [testList])
    
    let reminders = try! await withCheckedThrowingContinuation { continuation in
        eventStore.fetchReminders(matching: predicate) { reminders in
            continuation.resume(returning: reminders ?? [])
        }
    }
    
    print("📝 Found \(reminders.count) reminders in TEST_LIST:")
    for (i, reminder) in reminders.enumerated() {
        print("  \(i + 1). '\(reminder.title ?? "")' (ID: \(reminder.calendarItemIdentifier))")
        print("     Completed: \(reminder.isCompleted)")
        print("     Priority: \(reminder.priority)")
        if let notes = reminder.notes {
            print("     Notes: \(notes)")
        }
    }
}

Task {
    await debugTestList()
    exit(0)
}

RunLoop.main.run()