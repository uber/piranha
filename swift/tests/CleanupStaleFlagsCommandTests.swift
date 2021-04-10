//
//  File.swift
//  
//
//  Created by Chirag Ramani on 06/04/21.
//

import Foundation
@testable import PiranhaKit
import XCTest
import ArgumentParser

final class CleanupStaleFlagsCommandTest: XCTestCase {
    
    func test_commandName() {
        XCTAssertEqual(CleanupStaleFlagsCommand._commandName,
                       "cleanup-stale-flags")
    }
    
    func test_helpMessage() {
        let expectedHelpMessage = "USAGE: cleanup-stale-flags [--source-file <source-file>] [--config-file <config-file>] --flag <flag> [--group-name <group-name>] [--treated]\n\nOPTIONS:\n  -s, --source-file <source-file>\n                          Input Source File that needs to run piranha \n  -c, --config-file <config-file>\n                          Path of configuration file for Piranha \n  -f, --flag <flag>       Name of the stale flag \n  --group-name <group-name>\n  -t, --treated           If this option is supplied, the flag is treated,\n                          otherwise it is control. \n  -h, --help              Show help information.\n"
        XCTAssertEqual(CleanupStaleFlagsCommand.helpMessage(),
                       expectedHelpMessage)
    }
}
