import EventKit
import Foundation

public class RemindersService: ObservableObject {
    private let eventStore = EKEventStore()
    
    public init() {}
    
    public func requestPermissions() async -> Bool {
        do {
            let status = try await eventStore.requestFullAccessToReminders()
            return status
        } catch {
            return false
        }
    }
    
    public func fetchLists() async throws -> [ReminderList] {
        let calendars = eventStore.calendars(for: .reminder)
        
        return try await withThrowingTaskGroup(of: ReminderList.self) { group in
            for calendar in calendars {
                group.addTask {
                    let count = try await self.getReminderCount(for: calendar)
                    return ReminderList(
                        id: calendar.calendarIdentifier,
                        name: calendar.title,
                        color: self.extractColor(from: calendar),
                        count: UInt32(count)
                    )
                }
            }
            
            var lists: [ReminderList] = []
            for try await list in group {
                lists.append(list)
            }
            return lists.sorted { $0.name < $1.name }
        }
    }
    
    public func fetchReminders(for listId: String) async throws -> [Reminder] {
        guard let calendar = eventStore.calendar(withIdentifier: listId) else {
            throw RemError.DataAccessError(message: "List not found")
        }
        
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        return try await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                guard let reminders = reminders else {
                    continuation.resume(throwing: RemError.DataAccessError(message: "Failed to fetch reminders"))
                    return
                }
                
                let mappedReminders = reminders.map { ekReminder in
                    Reminder(
                        id: ekReminder.calendarItemIdentifier,
                        title: ekReminder.title ?? "",
                        notes: ekReminder.notes,
                        completed: ekReminder.isCompleted,
                        priority: UInt8(ekReminder.priority),
                        dueDate: ekReminder.dueDateComponents?.date?.ISO8601Format()
                    )
                }
                
                continuation.resume(returning: mappedReminders)
            }
        }
    }
    
    public func toggleReminder(_ reminderId: String) async throws {
        // First, find the reminder across all calendars
        let calendars = eventStore.calendars(for: .reminder)
        
        for calendar in calendars {
            let predicate = eventStore.predicateForReminders(in: [calendar])
            
            let reminders = try await withCheckedThrowingContinuation { continuation in
                eventStore.fetchReminders(matching: predicate) { reminders in
                    continuation.resume(returning: reminders ?? [])
                }
            }
            
            if let reminder = reminders.first(where: { $0.calendarItemIdentifier == reminderId }) {
                reminder.isCompleted = !reminder.isCompleted
                
                try eventStore.save(reminder, commit: true)
                return
            }
        }
        
        throw RemError.DataAccessError(message: "Reminder not found")
    }
    
    public func deleteReminder(_ reminderId: String) async throws {
        // Fetch reminders from all calendars to find the one with matching ID
        let calendars = eventStore.calendars(for: .reminder)
        
        for calendar in calendars {
            let predicate = eventStore.predicateForReminders(in: [calendar])
            
            let reminders = try await withCheckedThrowingContinuation { continuation in
                eventStore.fetchReminders(matching: predicate) { reminders in
                    continuation.resume(returning: reminders ?? [])
                }
            }
            
            if let reminder = reminders.first(where: { $0.calendarItemIdentifier == reminderId }) {
                try eventStore.remove(reminder, commit: true)
                return
            }
        }
        
        throw RemError.DataAccessError(message: "Reminder not found")
    }
    
    public func createReminder(_ newReminder: NewReminder) async throws {
        guard let calendar = eventStore.calendar(withIdentifier: newReminder.listId) else {
            throw RemError.DataAccessError(message: "List not found")
        }
        
        let reminder = EKReminder(eventStore: eventStore)
        reminder.title = newReminder.title
        reminder.notes = newReminder.notes
        reminder.calendar = calendar
        reminder.isCompleted = false
        reminder.priority = Int(newReminder.priority)
        
        // Handle due date if provided
        if let dueDateString = newReminder.dueDate, !dueDateString.isEmpty {
            // Try to parse the date string (you might want to implement more sophisticated date parsing)
            let formatter = ISO8601DateFormatter()
            if let date = formatter.date(from: dueDateString) {
                let components = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
                reminder.dueDateComponents = components
            }
        }
        
        try eventStore.save(reminder, commit: true)
    }
    
    private func getReminderCount(for calendar: EKCalendar) async throws -> Int {
        let predicate = eventStore.predicateForReminders(in: [calendar])
        
        return try await withCheckedThrowingContinuation { continuation in
            eventStore.fetchReminders(matching: predicate) { reminders in
                continuation.resume(returning: reminders?.count ?? 0)
            }
        }
    }
    
    private func extractColor(from calendar: EKCalendar) -> String {
        if let cgColor = calendar.cgColor {
            // Convert CGColor to hex string
            if let _ = cgColor.colorSpace,
               let components = cgColor.components,
               components.count >= 3 {
                let r = Int(components[0] * 255)
                let g = Int(components[1] * 255)
                let b = Int(components[2] * 255)
                return String(format: "#%02x%02x%02x", r, g, b)
            }
        }
        
        // Fallback colors
        return "#007AFF"
    }
}

// Note: ReminderList, Reminder, and RemError types are now generated by UniFFI
// and available from the RemCore module