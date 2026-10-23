// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "AppKitBridge",
    platforms: [.macOS(.v11)],
    products: [
        .library(name: "AppKitBridge", type: .static, targets: ["AppKitBridge"]),
    ],
    targets: [
        .target(name: "AppKitBridge", path: "Sources/AppKitBridge", publicHeadersPath: "include"),
    ]
)
