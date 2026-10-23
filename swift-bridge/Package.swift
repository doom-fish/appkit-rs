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
        .target(name: "AppKitExceptionShim", path: "Sources/AppKitExceptionShim", publicHeadersPath: "include"),
        .target(name: "AppKitBridge", dependencies: ["AppKitCallbackABI", "AppKitExceptionShim"], path: "Sources/AppKitBridge", publicHeadersPath: "include"),
    ]
)
