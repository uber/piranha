//
//  Copyright © Uber Technologies, Inc. All rights reserved.
//

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
