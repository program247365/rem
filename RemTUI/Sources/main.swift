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
            // Start single TUI session with async operation handling
            await runPersistentTUI(lists: lists, remindersService: remindersService)
            
        } catch {
            print("❌ Critical error starting TUI: \(error)")
            exit(1)
        }
    }
    
    // SIMPLE: Test with the original working TUI first
    private static func runPersistentTUI(lists: [ReminderList], remindersService: RemindersService) async {
        var currentLists = lists
        
        while true {
            do {
                let actions = try startTui(lists: currentLists)
                
                var shouldExit = false
                for action in actions {
                    switch action {
                    case .quit:
                        shouldExit = true
                        break
                        
                    case .selectList(let listId):
                        do {
                            let reminders = try await remindersService.fetchReminders(for: listId)
                            let reminderActions = try renderRemindersView(reminders: reminders)
                            
                            for reminderAction in reminderActions {
                                switch reminderAction {
                                case .quit:
                                    shouldExit = true
                                    break
                                case .back:
                                    break
                                case .toggleReminder(let reminderId):
                                    try await remindersService.toggleReminder(reminderId)
                                case .deleteReminder(let reminderId):
                                    try await remindersService.deleteReminder(reminderId)
                                case .createReminder(let newReminder):
                                    try await remindersService.createReminder(newReminder)
                                default:
                                    break
                                }
                            }
                        } catch {
                            // Handle error and continue
                        }
                        
                    case .refresh:
                        currentLists = try await remindersService.fetchLists()
                        
                    default:
                        break
                    }
                }
                
                if shouldExit {
                    break
                }
                
            } catch {
                print("❌ TUI error: \(error)")
                break
            }
        }
    }
}