// swift-tools-version:5.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Piranha",
    platforms: [
        .macOS(.v10_14),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-syntax.git", .exact("0.50100.0")),
    ],
    targets: [
        .target(
            name: "Piranha",
            dependencies: ["SwiftSyntax"],
            path: "src"),
        .testTarget(
            name: "PiranhaTests",
            dependencies: ["Piranha"],
            path: "tests"),
    ]
)
