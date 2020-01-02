//
//  Copyright © Uber Technologies, Inc. All rights reserved.
//

import Foundation
import SwiftSyntax

// groupname is optional. the other arguments are necessary
class StaleFlagCleanerLauncher: CommandLauncher {
    let command: Command = .cleanupStaleFlags

    func launch(_ args: [String]) throws {
        var sourceFile = URL(fileURLWithPath: "")
        var configFile = URL(fileURLWithPath: "")
        var flagName = ""
        var groupName = ""
        var isTreated = false

        for (index, argument) in args.enumerated() {
            switch index {
            case 2:
                configFile = URL(fileURLWithPath: argument)
            case 3:
                sourceFile = URL(fileURLWithPath: argument)
            case 4:
                flagName = argument
            case 5:
                isTreated = argument.elementsEqual("true") ? true : false
            case 6:
                groupName = argument
            default: break
            }
        }

        guard flagName.count > 0 else {
            // swiftlint:disable:next custom_rules
            print("Flag name is necessary to use the refactoring tool.")
            exit(-1)
        }

        let parsed = try SyntaxParser.parse(sourceFile)
        let cleaner = XPFlagCleaner(with: configFile, flag: flagName, behavior: isTreated, group: groupName)

        // swiftlint:disable:next custom_rules
        var refactoredOutput = cleaner.visit(parsed)
        if cleaner.deepClean() {
            cleaner.setNextPass()
            refactoredOutput = cleaner.visit(parsed)
        }

        // swiftlint:disable:next custom_rules
        print(refactoredOutput, terminator: "")
    }
}
