// swift-tools-version:5.3
// The swift-tools-version declares the minimum version of Swift required to build this package.
/**
 *    Copyright (c) 2019 Uber Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */


import PackageDescription

let package = Package(
    name: "Piranha",
    platforms: [
        .macOS(.v10_15),
    ],
    dependencies: [
        .package(name: "SwiftSyntax",
                 url: "https://github.com/apple/swift-syntax.git",
                 .exact("0.50500.0")),
        .package(url: "https://github.com/apple/swift-argument-parser",
                 from: "1.0.1"),
    ],
    targets: [
        .target(
            name: "Piranha",
            dependencies: ["PiranhaKit"]),
        .target(
            name: "PiranhaKit",
            dependencies: ["SwiftSyntax",
                           .product(name: "ArgumentParser",
                                    package: "swift-argument-parser")]),
        .testTarget(
            name: "PiranhaTests",
            dependencies: ["PiranhaKit"],
            path: "Tests",
            exclude: ["InputSampleFiles/"])
    ]
)
