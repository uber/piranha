//
//  Tests.swift
//  PiranhaTests
//
import Foundation
import XCTest
import Piranha

class PiranhaTests: XCTestCase {
    let dstPath = "tests/treated.swift"
    let srcPath = "tests/testfile.swift"
    let configPath = "piranha.properties"
    let flag = "test_experiment"
    let group = "group"

    func testFlags() {
        // TODO: create piranha target before uncommenting below
//       _ = runCleaner(["", "", configPath, srcPath, flag, "true", group])
    }
}
