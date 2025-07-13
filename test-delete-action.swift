#!/usr/bin/env swift

import Foundation
import EventKit

// Import our actual service (simulate the RemindersService)
let eventStore = EKEventStore()

func requestPermission() async -> Bool {
    return await withCheckedContinuation { continuation in
        eventStore.requestFullAccessToReminders { granted, error in
            continuation.resume(returning: granted)
        }
    }
}

func fetchReminders(for listId: String) async throws -> [(id: String, title: String)] {
    guard let calendar = eventStore.calendar(withIdentifier: listId) else {
        throw NSError(domain: "TestError", code: 1, userInfo: [NSLocalizedDescriptionKey: "List not found"])
    }
    
    let predicate = eventStore.predicateForReminders(in: [calendar])
    
    return try await withCheckedThrowingContinuation { continuation in
        eventStore.fetchReminders(matching: predicate) { reminders in
            guard let reminders = reminders else {
                continuation.resume(throwing: NSError(domain: "TestError", code: 2, userInfo: [NSLocalizedDescriptionKey: "Failed to fetch reminders"]))
                return
            }
            
            let mappedReminders = reminders.map { ekReminder in
                (id: ekReminder.calendarItemIdentifier, title: ekReminder.title ?? "")
            }
            
            continuation.resume(returning: mappedReminders)
        }
    }
}

func deleteReminder(_ reminderId: String) async throws {
    print("🗑️ DEBUG: Attempting to delete reminder with ID: \(reminderId)")
    
    let calendars = eventStore.calendars(for: .reminder)
    print("🗑️ DEBUG: Searching across \(calendars.count) calendars")
    
    for calendar in calendars {
        print("🗑️ DEBUG: Checking calendar: \(calendar.title)")
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        let reminders = try await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders ?? [])
            }
        }
        
        print("🗑️ DEBUG: Found \(reminders.count) reminders in \(calendar.title)")
        
        if let reminder = reminders.first(where: { $0.calendarItemIdentifier == reminderId }) {
            print("🗑️ DEBUG: Found matching reminder: '\(reminder.title ?? "")' with ID: \(reminder.calendarItemIdentifier)")
            do {
                try eventStore.remove(reminder, commit: true)
                print("🗑️ DEBUG: Successfully removed reminder from EventKit")
                return
            } catch {
                print("🗑️ DEBUG: Failed to remove reminder: \(error)")
                throw error
            }
        }
    }
    
    print("🗑️ DEBUG: No reminder found with ID: \(reminderId)")
    throw NSError(domain: "TestError", code: 3, userInfo: [NSLocalizedDescriptionKey: "Reminder not found"])
}

func testDeleteFlow() async {
    print("🧪 Testing delete flow for TEST_LIST...")
    
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
    
    do {
        // Fetch reminders (like TUI does)
        let reminders = try await fetchReminders(for: testList.calendarIdentifier)
        print("📝 Fetched \(reminders.count) reminders from TEST_LIST:")
        
        for (i, reminder) in reminders.enumerated() {
            print("  \(i). '\(reminder.title)' (ID: \(reminder.id))")
        }
        
        // Test deletion of first reminder (simulate TUI selection)
        if let firstReminder = reminders.first {
            print("\\n🗑️ Attempting to delete: '\(firstReminder.title)' (ID: \(firstReminder.id))")
            
            try await deleteReminder(firstReminder.id)
            print("✅ Delete action completed!")
            
            // Check if it's actually gone
            print("\\n🔄 Checking if reminder is actually deleted...")
            let remindersAfter = try await fetchReminders(for: testList.calendarIdentifier)
            print("📝 Reminders after deletion: \(remindersAfter.count)")
            
            if remindersAfter.count < reminders.count {
                print("✅ SUCCESS: Reminder was actually deleted!")
            } else {
                print("❌ FAILURE: Reminder still exists after deletion")
                for reminder in remindersAfter {
                    print("  - '\(reminder.title)' (ID: \(reminder.id))")
                }
            }
        }
        
    } catch {
        print("❌ Error: \(error)")
    }
}

Task {
    await testDeleteFlow()
    exit(0)
}

RunLoop.main.run()