/**
 *    Copyright (c) 2021 Uber Technologies, Inc.
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

final class PiranhaConfigProviderDefaultImplTests: XCTestCase {
    
    private let sut = PiranhaConfigProviderDefaultImpl()
    private let configURL = FileManager.default.temporaryDirectory.appendingPathComponent("config.json")
    
    override func tearDown() {
        super.tearDown()
        try? FileManager.default.removeItem(at: configURL)
    }
    
    func test_invalidConfig_methodNameMissing() {
        // given
        try! invalidConfig_methodNameMissing.write(to: configURL,
                                                   atomically: false,
                                                   encoding: .utf8)
        do {
            // when
            _ = try sut.config(fromFileURL: configURL)
            // then
            XCTFail("Invalid config should be constructed")
        } catch let error as ValidationError {
            XCTAssertEqual(error.message, "Invalid configuration")
        } catch let error {
            // then
            XCTFail("No error is expected for valid input but received: \(error.localizedDescription)")
        }
    }
    
    func test_invalidConfig_flagIndexMissing() {
        // given
        try! invalidConfig_flagIndexMissing.write(to: configURL,
                                                  atomically: false,
                                                  encoding: .utf8)
        do {
            // when
            _ = try sut.config(fromFileURL: configURL)
            // then
            XCTFail("Invalid config should be constructed")
        } catch let error as ValidationError {
            XCTAssertEqual(error.message, "Invalid configuration")
        } catch let error {
            // then
            XCTFail("No error is expected for valid input but received: \(error.localizedDescription)")
        }
    }
    
    func test_invalidConfig_flagTypeMissing() {
        // given
        try! invalidConfig_flagTypeMissing.write(to: configURL,
                                                 atomically: false,
                                                 encoding: .utf8)
        do {
            // when
            _ = try sut.config(fromFileURL: configURL)
            // then
            XCTFail("Invalid config should be constructed")
        } catch let error as ValidationError {
            XCTAssertEqual(error.message, "Invalid configuration")
        } catch let error {
            // then
            XCTFail("No error is expected for valid input but received: \(error.localizedDescription)")
        }
    }
    
    func test_validConfig() {
        // given
        try! validConfig.write(to: configURL,
                               atomically: false,
                               encoding: .utf8)
        
        // when
        let config = try! sut.config(fromFileURL: configURL)
        
        // then
        XCTAssertEqual(config.methods.count, 7)
        XCTAssertEqual(config.methods[0], PiranhaKit.Method(methodName: "isTreated",
                                                            flagType: .treated,
                                                            flagIndex: 0,
                                                            groupIndex: nil))
        XCTAssertEqual(config.methods[1], PiranhaKit.Method(methodName: "isInControlGroup",
                                                            flagType: .control,
                                                            flagIndex: 0,
                                                            groupIndex: nil))
        XCTAssertEqual(config.methods[2], PiranhaKit.Method(methodName: "isInTreatmentGroup",
                                                            flagType: .treated,
                                                            flagIndex: 1,
                                                            groupIndex: 0))
        XCTAssertEqual(config.methods[3], PiranhaKit.Method(methodName: "addTreatedExperiment",
                                                            flagType: .testing,
                                                            flagIndex: 0,
                                                            groupIndex: nil))
        XCTAssertEqual(config.methods[4], PiranhaKit.Method(methodName: "removeTreatedExperiment",
                                                            flagType: .testing,
                                                            flagIndex: 0,
                                                            groupIndex: nil))
        XCTAssertEqual(config.methods[5], PiranhaKit.Method(methodName: "addExperimentParameter",
                                                            flagType: .testing,
                                                            flagIndex: 0,
                                                            groupIndex: nil))
        XCTAssertEqual(config.methods[6], PiranhaKit.Method(methodName: "experimentParameter",
                                                            flagType: .testing,
                                                            flagIndex: 1,
                                                            groupIndex: nil))
    }
    
    // MARK: Configs
    
    private let invalidConfig_methodNameMissing =
        """
    {
    "methodProperties": [
        {
            "methodName": "isTreated",
            "flagType": "treated",
            "flagIndex": 0
        },
        {
            "flagType": "control",
            "flagIndex": 0
        },
        {
            "methodName": "isInTreatmentGroup",
            "flagType": "treated",
            "flagIndex": 1,
            "groupIndex": 0
        },
        {
            "methodName": "addTreatedExperiment",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "addExperimentParameter",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "experimentParameter",
            "flagType": "testing",
            "flagIndex": 1
        }
    ]
    }
    """
    
    private let invalidConfig_flagIndexMissing =
        """
    {
    "methodProperties": [
        {
            "methodName": "isTreated",
            "flagType": "treated",
            "flagIndex": 0
        },
        {
            "methodName": "isInControlGroup",
            "flagType": "control",
        },
        {
            "methodName": "isInTreatmentGroup",
            "flagType": "treated",
            "groupIndex": 0
        },
        {
            "methodName": "addTreatedExperiment",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "addExperimentParameter",
            "flagType": "testing",
        },
        {
            "methodName": "experimentParameter",
            "flagType": "testing",
            "flagIndex": 1
        }
    ]
    }
    """
    
    private let invalidConfig_flagTypeMissing =
        """
    {
    "methodProperties": [
        {
            "methodName": "isTreated",
            "flagType": "treated",
            "flagIndex": 0
        },
        {
            "methodName": "isInControlGroup",
            "flagIndex": 0
        },
        {
            "methodName": "isInTreatmentGroup",
            "flagType": "treated",
            "groupIndex": 0
        },
        {
            "methodName": "addTreatedExperiment",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "addExperimentParameter",
            "flagIndex": 0
        },
        {
            "methodName": "experimentParameter",
            "flagType": "testing",
            "flagIndex": 1
        }
    ]
    }
    """
    
    private let validConfig =
        """
    {
    "methodProperties": [
        {
            "methodName": "isTreated",
            "flagType": "treated",
            "flagIndex": 0
        },
        {
            "methodName": "isInControlGroup",
            "flagType": "control",
            "flagIndex": 0
        },
        {
            "methodName": "isInTreatmentGroup",
            "flagType": "treated",
            "flagIndex": 1,
            "groupIndex": 0
        },
        {
            "methodName": "addTreatedExperiment",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "removeTreatedExperiment",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "addExperimentParameter",
            "flagType": "testing",
            "flagIndex": 0
        },
        {
            "methodName": "experimentParameter",
            "flagType": "testing",
            "flagIndex": 1
        }
    ]
    }
    """
}

