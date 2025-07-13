import Foundation
import RemTUIKit
// TODO: Import RemCore once UniFFI bindings are generated

@main
struct RemTUIApp {
    static func main() async {
        let permissionManager = PermissionManager()
        let remindersService = RemindersService()
        
        print("🍎 Rem - Fast TUI for Apple Reminders")
        print("🔧 Checking permissions...")
        
        // Check permissions
        let status = permissionManager.checkPermissionStatus()
        switch status {
        case .notDetermined:
            print("📋 Requesting Reminders access...")
            let granted = await permissionManager.requestPermissions()
            if !granted {
                print("❌ Permission denied. Reminders access is required.")
                print("Please run the app again and grant permission, or enable it manually in System Settings.")
                exit(1)
            }
            print("✅ Permission granted!")
            
        case .denied, .restricted:
            print("❌ Reminders access is denied or restricted.")
            permissionManager.showPermissionGuidance()
            exit(1)
            
        case .fullAccess, .writeOnly:
            print("✅ Reminders access already granted")
            
        @unknown default:
            print("⚠️ Unknown permission status: \(status.rawValue)")
            exit(1)
        }
        
        do {
            print("📱 Loading your reminder lists...")
            let lists = try await remindersService.fetchLists()
            
            if lists.isEmpty {
                print("📭 No reminder lists found in your Reminders app.")
                print("Please create some lists and reminders in the Reminders app first.")
                exit(0)
            }
            
            print("✅ Found \(lists.count) reminder lists")
            for list in lists {
                print("   • \(list.name) (\(list.count) reminders)")
            }
            
            print("🚀 Starting TUI interface...")
            
            // Main TUI loop - restart seamlessly after actions
            await runTUILoop(lists: lists, remindersService: remindersService)
            
        } catch {
            print("❌ Error: \(error)")
            if let remError = error as? RemError {
                switch remError {
                case .PermissionDenied:
                    print("Permission was denied. Please enable Reminders access in System Settings.")
                case .DataAccessError(let message):
                    print("Data access error: \(message)")
                case .TuiError(let message):
                    print("TUI error: \(message)")
                }
            }
            exit(1)
        }
    }
    
    private static func runTUILoop(lists: [ReminderList], remindersService: RemindersService) async {
        var currentLists = lists
        var currentListId: String? = nil  // Track which list we're viewing
        
        while true {
            do {
                // Start TUI and get actions
                let actions = try startTui(lists: currentLists)
                
                // Process actions and check if we should continue
                let (shouldContinue, newListId) = await processActions(actions, remindersService: remindersService, lists: currentLists, currentListId: currentListId)
                
                if !shouldContinue {
                    break
                }
                
                // Update current list ID if changed
                currentListId = newListId
                
                // Refresh lists data for next iteration
                currentLists = try await remindersService.fetchLists()
                
                // If we're viewing a specific list, refresh that view for next iteration
                if let listId = currentListId {
                    let refreshedReminders = try await remindersService.fetchReminders(for: listId)
                    try setReminders(reminders: refreshedReminders)
                }
                
            } catch {
                print("❌ Error in TUI: \(error)")
                break
            }
        }
    }
    
    private static func processActions(
        _ actions: [TuiAction],
        remindersService: RemindersService,
        lists: [ReminderList],
        currentListId: String?
    ) async -> (shouldContinue: Bool, newListId: String?) {
        var updatedListId = currentListId
        
        for action in actions {
            switch action {
            case .quit:
                print("👋 Goodbye!")
                return (false, nil)  // Exit the TUI loop
                
            case .selectList(let listId):
                do {
                    print("📋 Loading reminders for selected list...")
                    let reminders = try await remindersService.fetchReminders(for: listId)
                    let reminderActions = try renderRemindersView(reminders: reminders)
                    updatedListId = listId  // Track which list we're now viewing
                    
                    // Process actions from the reminders view
                    for reminderAction in reminderActions {
                        switch reminderAction {
                        case .deleteReminder(let reminderId):
                            try await remindersService.deleteReminder(reminderId)
                            print("🗑️ Reminder deleted")
                            
                        case .toggleReminder(let reminderId):
                            try await remindersService.toggleReminder(reminderId)
                            print("✅ Reminder toggled")
                            
                        case .back:
                            updatedListId = nil  // Going back to lists
                            
                        case .quit:
                            return (false, nil)  // Exit completely
                            
                        default:
                            break
                        }
                    }
                } catch {
                    print("❌ Error loading reminders: \(error)")
                }
                
            case .toggleReminder(let reminderId):
                do {
                    try await remindersService.toggleReminder(reminderId)
                    print("✅ Reminder toggled")
                } catch {
                    print("❌ Error toggling reminder: \(error)")
                }
                
            case .deleteReminder(_):
                // This should no longer happen since deletes are handled in reminders view
                break
                
            case .back:
                // Going back to lists view
                updatedListId = nil
                
            case .refresh:
                // TUI will restart with refreshed data
                break
            }
        }
        return (true, updatedListId)  // Continue the TUI loop
    }
}