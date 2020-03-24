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

// groupname is optional. the other arguments are necessary
public class StaleFlagCleanerLauncher: CommandLauncher {
    let command: Command = .cleanupStaleFlags
    public init() {
    }

    public func launch(_ args: [String]) throws {
        runCleaner(args)
    }
}

public func runCleaner(_ args: [String]) -> String {
    var source = ""
    var config = ""
    var flagName = ""
    var groupName = ""
    var isTreated = false

    for (index, argument) in args.enumerated() {
        switch index {
        case 2:
            config = argument
        case 3:
            source = argument
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
    do {

        let sourceFile = URL(fileURLWithPath: source)
        let configFile = URL(fileURLWithPath: config)

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
        return refactoredOutput.description
    } catch {
        fatalError(error.localizedDescription)
    }
}
