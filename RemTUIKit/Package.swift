// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "RemTUIKit",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "RemTUIKit", targets: ["RemTUIKit"])
    ],
    targets: [
        .target(
            name: "RemCoreFFI",
            dependencies: [],
            path: "Sources/RemCoreFFI",
            publicHeadersPath: "include"
        ),
        .target(
            name: "RemTUIKit",
            dependencies: ["RemCoreFFI"],
            path: "Sources/RemTUIKit",
            exclude: ["librem_core.dylib"],
            linkerSettings: [
                .unsafeFlags(["-L", "Sources/RemTUIKit"]),
                .linkedLibrary("rem_core")
            ]
        ),
        .testTarget(
            name: "RemTUIKitTests", 
            dependencies: ["RemTUIKit"],
            path: "Tests/RemTUIKitTests"
        )
    ]
)