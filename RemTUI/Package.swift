// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "RemTUI",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(path: "../RemTUIKit")
    ],
    targets: [
        .executableTarget(
            name: "RemTUI",
            dependencies: ["RemTUIKit"],
            path: "Sources",
            linkerSettings: [
                .unsafeFlags(["-L", "../RemTUIKit/Sources/RemTUIKit"]),
                .linkedLibrary("rem_core")
            ]
        )
    ]
)