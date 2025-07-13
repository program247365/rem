#!/usr/bin/env swift

import Foundation
import EventKit

// Import our actual service types (we'll simulate them)
let eventStore = EKEventStore()

func requestPermission() async -> Bool {
    return await withCheckedContinuation { continuation in
        eventStore.requestFullAccessToReminders { granted, error in
            continuation.resume(returning: granted)
        }
    }
}

// Simulate the RemindersService createReminder method
func createReminder(title: String, notes: String?, dueDate: String?, listId: String, priority: UInt8) async throws {
    guard let calendar = eventStore.calendar(withIdentifier: listId) else {
        throw NSError(domain: "TestError", code: 1, userInfo: [NSLocalizedDescriptionKey: "List not found"])
    }
    
    let reminder = EKReminder(eventStore: eventStore)
    reminder.title = title
    reminder.notes = notes
    reminder.calendar = calendar
    reminder.isCompleted = false
    reminder.priority = Int(priority)
    
    // Handle due date if provided
    if let dueDateString = dueDate, !dueDateString.isEmpty {
        let formatter = ISO8601DateFormatter()
        if let date = formatter.date(from: dueDateString) {
            let components = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
            reminder.dueDateComponents = components
        }
    }
    
    try eventStore.save(reminder, commit: true)
}

func testCreateWorkflow() async {
    print("🧪 Testing complete create reminder workflow...")
    
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
    
    print("✅ Found TEST_LIST with ID: \(testList.calendarIdentifier)")
    
    // Test form data (simulating what would come from TUI form)
    let formData = [
        ("Simple Reminder", nil, nil, UInt8(0)),
        ("Reminder with Notes", "This is a test note from the form", nil, UInt8(3)),
        ("High Priority Reminder", "Important task", "2024-12-31T23:59:59Z", UInt8(9))
    ]
    
    for (i, (title, notes, dueDate, priority)) in formData.enumerated() {
        print("\\n📝 Creating reminder \(i + 1): '\(title)'")
        
        do {
            try await createReminder(
                title: title,
                notes: notes,
                dueDate: dueDate,
                listId: testList.calendarIdentifier,
                priority: priority
            )
            print("✅ Successfully created: '\(title)'")
        } catch {
            print("❌ Failed to create '\(title)': \(error)")
        }
    }
    
    // Verify all reminders were created
    print("\\n🔍 Verifying created reminders...")
    let predicate = eventStore.predicateForReminders(in: [testList])
    
    let reminders = try! await withCheckedThrowingContinuation { continuation in
        eventStore.fetchReminders(matching: predicate) { reminders in
            continuation.resume(returning: reminders ?? [])
        }
    }
    
    print("📝 Total reminders in TEST_LIST: \(reminders.count)")
    for reminder in reminders {
        print("  - '\(reminder.title ?? "")' (Priority: \(reminder.priority), Notes: '\(reminder.notes ?? "None")')")
    }
    
    print("\\n✅ Create workflow test completed!")
}

Task {
    await testCreateWorkflow()
    exit(0)
}

RunLoop.main.run()