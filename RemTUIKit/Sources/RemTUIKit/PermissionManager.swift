import EventKit
import Foundation

public class PermissionManager {
    private let eventStore = EKEventStore()
    
    public init() {}
    
    public func checkPermissionStatus() -> EKAuthorizationStatus {
        return EKEventStore.authorizationStatus(for: .reminder)
    }
    
    public func requestPermissions() async -> Bool {
        do {
            return try await eventStore.requestFullAccessToReminders()
        } catch {
            return false
        }
    }
    
    public func showPermissionGuidance() {
        print("""
        📝 Reminders Access Required
        
        This app needs permission to access your Reminders.
        
        If the permission dialog doesn't appear:
        1. Open System Settings → Privacy & Security → Reminders
        2. Enable access for this application
        3. Restart the app
        
        Press any key to continue...
        """)
        _ = readLine()
    }
    
    public func printPermissionStatus() {
        let status = checkPermissionStatus()
        switch status {
        case .notDetermined:
            print("⚠️  Permission not determined - will request access")
        case .denied:
            print("❌ Permission denied - please enable in System Settings")
        case .restricted:
            print("🔒 Permission restricted - check parental controls")
        case .fullAccess:
            print("✅ Full access granted")
        case .writeOnly:
            print("📝 Write-only access granted")
        @unknown default:
            print("❓ Unknown permission status: \(status.rawValue)")
        }
    }
}