#!/usr/bin/env swift

import Foundation
import EventKit

// Test script to verify create reminder functionality works
// Run this to test EventKit creation without the TUI

let eventStore = EKEventStore()

func requestPermission() async -> Bool {
    return await withCheckedContinuation { continuation in
        eventStore.requestFullAccessToReminders { granted, error in
            continuation.resume(returning: granted)
        }
    }
}

func testCreateReminder() async {
    print("🧪 Testing reminder creation functionality...")
    
    // Request permission
    let granted = await requestPermission()
    guard granted else {
        print("❌ Permission denied")
        return
    }
    
    print("✅ Permission granted")
    
    // Get TEST_LIST calendar
    let calendars = eventStore.calendars(for: .reminder)
    guard let testList = calendars.first(where: { $0.title == "TEST_LIST" }) else {
        print("❌ TEST_LIST not found")
        return
    }
    
    print("✅ Found TEST_LIST with ID: \(testList.calendarIdentifier)")
    
    // Create a test reminder
    let reminder = EKReminder(eventStore: eventStore)
    reminder.title = "Test Created Reminder"
    reminder.notes = "This reminder was created by the test script"
    reminder.calendar = testList
    reminder.isCompleted = false
    reminder.priority = 5
    
    do {
        try eventStore.save(reminder, commit: true)
        print("✅ Successfully created reminder: '\(reminder.title!)' with ID: \(reminder.calendarItemIdentifier)")
        
        // Verify it was created by fetching reminders
        let predicate = eventStore.predicateForReminders(in: [testList])
        let reminders = try! await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders ?? [])
            }
        }
        
        print("📝 Total reminders in TEST_LIST after creation: \(reminders.count)")
        if let newReminder = reminders.first(where: { $0.title == "Test Created Reminder" }) {
            print("🎯 Found newly created reminder: '\(newReminder.title ?? "")' (ID: \(newReminder.calendarItemIdentifier))")
            print("   Notes: '\(newReminder.notes ?? "No notes")'")
            print("   Priority: \(newReminder.priority)")
            print("   Completed: \(newReminder.isCompleted)")
        } else {
            print("❌ Could not find newly created reminder")
        }
        
    } catch {
        print("❌ Failed to create reminder: \(error)")
    }
}

// Run the test
Task {
    await testCreateReminder()
    exit(0)
}

RunLoop.main.run()