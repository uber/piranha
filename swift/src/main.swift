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

import Foundation
import SwiftSyntax

// Create a CommandLauncher and add it here if you want to add a new Piranha command to the Swift executable.
let commandLaunchers: [CommandLauncher] = [
    StaleFlagCleanerLauncher(),
]

private func showError(_ text: String) {
    print("ERROR: \(text)")
}

guard CommandLine.arguments.count > 1 else {
    showError("Command must be specified.")
    exit(-1)
}
let command = CommandLine.arguments[1]

let filteredCommandLaunchers = commandLaunchers.filter { (commandLauncher: CommandLauncher) -> Bool in
    return commandLauncher.command.rawValue == command
}

guard let firstCommandLauncher = filteredCommandLaunchers.first else {
    showError("Unhandled command.")
    exit(-1)
}

do {
    try firstCommandLauncher.launch(CommandLine.arguments)
} catch {
    showError("Failed to launch command.")
}
