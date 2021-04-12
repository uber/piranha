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
import ArgumentParser
import SwiftSyntax

public struct PiranhaCommand: ParsableCommand {
    
    public init() { }
    
    public static var configuration = CommandConfiguration(subcommands: [CleanupStaleFlagsCommand.self],
                                                           defaultSubcommand: CleanupStaleFlagsCommand.self)
}

struct CleanupStaleFlagsCommand: ParsableCommand {
    
    static var _commandName: String = "cleanup-stale-flags"
    
    @Option(name: [.customShort("s"),
                   .long],
            help: "Input source file for Piranha",
            transform: { URL(fileURLWithPath: $0)})
    var sourceFile: URL?
    
    @Option(name: [.customShort("c"),
                   .long],
            help: "Path of configuration file for Piranha",
            transform: { URL(fileURLWithPath: $0)})
    var configFile: URL?
    
    @Option(name: [.customShort("f"),
                   .long],
            help: "Name of the stale flag")
    var flag: String
    
    @Option
    var groupName: String?
    
    @Flag(name: [.customShort("t"),
                 .long],
          help: "If this option is supplied, the flag is treated, otherwise it is control.")
    var treated = false
    
    
    func validate() throws {
        try CleanupStaleFlagsCommandInputValidator().validateInput(sourceFileURL: sourceFile,
                                                                   configFileURL: configFile,
                                                                   flag: flag,
                                                                   fileManager: FileManager.default)
    }
    
    func run() throws {
        guard let sourceFileURL = sourceFile,
              let configFileURL = configFile else {
            // This ideally shouldn't happen since validation runs before the run phase that ensures that invariant is satisfied.
            throw ValidationError("Invalid file paths")
        }
        let parsed = try SyntaxParser.parse(sourceFileURL)
        let cleaner = XPFlagCleaner(with: configFileURL,
                                    flag: flag,
                                    behavior: treated,
                                    group: groupName ?? "")
        
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
