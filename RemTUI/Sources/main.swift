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
        
        while true {
            do {
                // Start TUI and get actions
                let actions = try startTui(lists: currentLists)
                
                // Process actions and check if we should continue
                let shouldContinue = await processActions(actions, remindersService: remindersService, lists: currentLists)
                
                if !shouldContinue {
                    break
                }
                
                // Refresh lists data for next iteration
                currentLists = try await remindersService.fetchLists()
                
            } catch {
                print("❌ Error in TUI: \(error)")
                break
            }
        }
    }
    
    private static func processActions(
        _ actions: [TuiAction],
        remindersService: RemindersService,
        lists: [ReminderList]
    ) async -> Bool {
        for action in actions {
            switch action {
            case .quit:
                print("👋 Goodbye!")
                return false  // Exit the TUI loop
                
            case .selectList(let listId):
                do {
                    print("📋 Loading reminders for selected list...")
                    let reminders = try await remindersService.fetchReminders(for: listId)
                    let _ = try renderRemindersView(reminders: reminders)
                    // TUI will restart and show the reminders view
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
                
            case .deleteReminder(let reminderId):
                do {
                    try await remindersService.deleteReminder(reminderId)
                    print("🗑️ Reminder deleted")
                } catch {
                    print("❌ Error deleting reminder: \(error)")
                }
                
            case .back:
                // TUI will restart and show the lists view
                break
                
            case .refresh:
                // TUI will restart with refreshed data
                break
            }
        }
        return true  // Continue the TUI loop
    }
}