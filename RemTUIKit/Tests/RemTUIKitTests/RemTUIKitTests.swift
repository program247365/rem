import XCTest
@testable import RemTUIKit

final class RemTUIKitTests: XCTestCase {
    func testPermissionManager() {
        let permissionManager = PermissionManager()
        // Basic initialization test
        XCTAssertNotNil(permissionManager)
    }
    
    func testRemindersService() {
        let remindersService = RemindersService()
        // Basic initialization test
        XCTAssertNotNil(remindersService)
    }
}