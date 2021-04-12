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

final class CleanupStaleFlagsCommandInputValidatorTest: XCTestCase {
    
    private let sut = CleanupStaleFlagsCommandInputValidator()
    private let fileManager = MockFileManager()
    
    func test_validateInput_invalidSourceFileURL() {
        // given
        fileManager.fileExistsHandler = { _ in false }
        
        // when
        do {
            try sut.validateInput(sourceFileURL: URL(fileURLWithPath: "sourceFileURLPath"),
                                  configFileURL: URL(fileURLWithPath: "configFileURLPath"),
                                  flag: "",
                                  fileManager: fileManager)
        } catch let error as ValidationError {
            // then
            XCTAssertEqual(error.message,
                           "Please provide valid source file path")
        } catch let error {
            // then
            XCTFail("Only ValidatorError is expected from CleanupStaleFlagsCommandInputValidator but receieved: \(error.localizedDescription)")
        }
    }
    
    func test_validateInput_invalidConfigFileURL() {
        // given
        fileManager.fileExistsHandler = { path in
            if path == "sourceFileURLPath" { return true }
            return false
        }
        
        // when
        do {
            try sut.validateInput(sourceFileURL: URL(fileURLWithPath: "sourceFileURLPath"),
                                  configFileURL: nil,
                                  flag: "",
                                  fileManager: fileManager)
        } catch let error as ValidationError {
            // then
            XCTAssertEqual(error.message,
                           "Please provide valid config file path")
        } catch let error {
            // then
            XCTFail("Only ValidatorError is expected from CleanupStaleFlagsCommandInputValidator but receieved: \(error.localizedDescription)")
        }
    }
    
    func test_validateInput_validFileURLs_emptyFlagName() {
        // given
        fileManager.fileExistsHandler = { _ in true }
        
        // when
        do {
            try sut.validateInput(sourceFileURL: URL(fileURLWithPath: "sourceFileURLPath"),
                                  configFileURL: URL(fileURLWithPath: "configFileURLPath"),
                                  flag: "",
                                  fileManager: fileManager)
        } catch let error as ValidationError {
            // then
            XCTAssertEqual(error.message,
                           "Please provide valid flag name")
        } catch let error {
            // then
            XCTFail("Only ValidatorError is expected from CleanupStaleFlagsCommandInputValidator but receieved: \(error.localizedDescription)")
        }
    }
    
    func test_validateInput_validFileURLs_nonEmptyFlagName() {
        // given
        fileManager.fileExistsHandler = { _ in true }
        
        // when
        do {
            try sut.validateInput(sourceFileURL: URL(fileURLWithPath: "sourceFileURLPath"),
                                  configFileURL: URL(fileURLWithPath: "configFileURLPath"),
                                  flag: "flagName",
                                  fileManager: fileManager)
        } catch let error as ValidationError {
            // then
            XCTFail("No error is expected for valid input but received: \(error.message)")
        } catch let error {
            // then
            XCTFail("No error is expected for valid input but receieved: \(error.localizedDescription)")
        }
    }
}


private class MockFileManager: FileManager {
    
    var fileExistsHandler: ((_ path: String) -> Bool)!
    
    override func fileExists(atPath path: String) -> Bool {
        fileExistsHandler(URL(string: path)!.lastPathComponent)
    }
}
