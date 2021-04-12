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
@testable import PiranhaKit
import XCTest
import ArgumentParser

final class CleanupStaleFlagsCommandTest: XCTestCase {
    
    func test_commandName() {
        XCTAssertEqual(CleanupStaleFlagsCommand._commandName,
                       "cleanup-stale-flags")
    }
    
    func test_helpMessage() {
        let expectedHelpMessage = "USAGE: cleanup-stale-flags [--source-file <source-file>] [--config-file <config-file>] --flag <flag> [--group-name <group-name>] [--treated]\n\nOPTIONS:\n  -s, --source-file <source-file>\n                          Input source file for Piranha \n  -c, --config-file <config-file>\n                          Path of configuration file for Piranha \n  -f, --flag <flag>       Name of the stale flag \n  --group-name <group-name>\n  -t, --treated           If this option is supplied, the flag is treated,\n                          otherwise it is control. \n  -h, --help              Show help information.\n"
        XCTAssertEqual(CleanupStaleFlagsCommand.helpMessage(),
                       expectedHelpMessage)
    }
}
