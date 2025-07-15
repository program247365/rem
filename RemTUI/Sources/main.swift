import Foundation
import RemTUIKit

@main
struct RemTUIApp {
    static func main() async {
        // Load data quickly first, then start TUI
        do {
            print("🚀 Starting Rem...")
            
            let permissionManager = PermissionManager()
            let remindersService = RemindersService()
            
            print("🔧 Checking permissions...")
            let status = permissionManager.checkPermissionStatus()
            
            switch status {
            case .notDetermined:
                print("📋 Requesting permissions...")
                let granted = await permissionManager.requestPermissions()
                if !granted {
                    print("❌ Permission denied. Please enable Reminders access in System Settings.")
                    exit(1)
                }
                
            case .denied, .restricted:
                print("❌ Reminders access is denied. Please enable it in System Settings.")
                exit(1)
                
            case .fullAccess, .writeOnly:
                print("✅ Permissions verified")
                
            @unknown default:
                print("⚠️ Unknown permission status")
                exit(1)
            }
            
            print("📱 Loading reminder lists...")
            let lists = try await remindersService.fetchLists()
            
            if lists.isEmpty {
                print("📭 No reminder lists found. Please create some lists in the Reminders app first.")
                exit(0)
            }
            
            print("✅ Found \(lists.count) lists. Starting TUI...")
            // Start persistent TUI session with proper bidirectional communication
            await runPersistentTUI(lists: lists, remindersService: remindersService)
            
        } catch {
            print("❌ Critical error starting TUI: \(error)")
            exit(1)
        }
    }
    
    // Truly persistent TUI that handles actions and continues running
    private static func runPersistentTUI(lists: [ReminderList], remindersService: RemindersService) async {
        var currentLists = lists
        var isRunning = true
        
        do {
            // Initialize the persistent TUI
            var actions = try runPersistentTui(lists: currentLists)
            
            // Main communication loop
            while isRunning {
                for action in actions {
                    switch action {
                    case .quit:
                        try shutdownTui()
                        isRunning = false
                        return
                        
                    case .selectList(let listId):
                        do {
                            let reminders = try await remindersService.fetchReminders(for: listId)
                            try setReminders(reminders: reminders)
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .globalSearch(let query):
                        do {
                            let (allReminders, listNames) = try await remindersService.searchAllReminders(query: query)
                            try setGlobalReminders(reminders: allReminders, listNames: listNames)
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .toggleReminder(let reminderId):
                        do {
                            try await remindersService.toggleReminder(reminderId)
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .deleteReminder(let reminderId):
                        do {
                            try await remindersService.deleteReminder(reminderId)
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .createReminder(let newReminder):
                        do {
                            try await remindersService.createReminder(newReminder)
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .refresh:
                        do {
                            currentLists = try await remindersService.fetchLists()
                        } catch {
                            // Error handling - TUI will show appropriate status
                        }
                        
                    case .back:
                        // No specific action needed - TUI handles navigation
                        break
                        
                    case .toggleCompletedVisibility:
                        // No specific action needed - TUI handles this internally
                        break
                        
                    case .showLoading(_):
                        // No specific action needed - just status
                        break
                        
                    case .dataLoaded:
                        // No specific action needed - just status
                        break
                    }
                }
                
                // Continue the TUI and get the next set of actions
                if isRunning {
                    actions = try continuePersistentTui()
                }
            }
            
        } catch {
            // Attempt graceful shutdown
            do {
                try shutdownTui()
            } catch {
                // Silent failure - TUI is shutting down anyway
            }
            exit(1)
        }
    }
}