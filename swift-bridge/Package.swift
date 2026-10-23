// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "AppKitBridge",
    platforms: [.macOS(.v12)],
    products: [
        .library(name: "AppKitBridge", type: .static, targets: ["AppKitBridge"]),
    ],
    targets: [
        .target(name: "AppKitCallbackABI", path: "Sources/AppKitCallbackABI", publicHeadersPath: "include"),
        .target(name: "AppKitBridge", dependencies: ["AppKitCallbackABI"], path: "Sources/AppKitBridge", publicHeadersPath: "include"),
    ]
)
