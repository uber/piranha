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
import SwiftSyntax
@testable import PiranhaKit
import XCTest

final class ConditionElementListSyntaxExtensionsTests: XCTestCase {
    
    func test_removeLastTrailingCommaIfNeeded_singleCondition_trailingCommaPresent() {
        // given
        var result = SyntaxFactory.makeBlankConditionElementList()
        let condition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("x")._syntaxNode,
                                                           trailingComma: nil)
        result = result.appending(.init({ (builder) in
            builder.useCondition(condition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken())
        }))
        XCTAssertEqual(result.description, "x,")
        
        // when
        result.removeLastTrailingCommaIfNeeded()
        
        // then
        XCTAssertEqual(result.description, "x")
    }
    
    func test_removeLastTrailingCommaIfNeeded_singleCondition_trailingCommaNotPresent() {
        // given
        var result = SyntaxFactory.makeBlankConditionElementList()
        let condition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("x")._syntaxNode,
                                                           trailingComma: nil)
        result = result.appending(.init({ (builder) in
            builder.useCondition(condition._syntaxNode)
        }))
        XCTAssertEqual(result.description, "x")
        
        // when
        result.removeLastTrailingCommaIfNeeded()
        
        // then
        XCTAssertEqual(result.description, "x")
    }
    
    func test_removeLastTrailingCommaIfNeeded_multipleConditions_trailingCommaPresent() {
        // given
        var result = SyntaxFactory.makeBlankConditionElementList()
        let xCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("x")._syntaxNode,
                                                            trailingComma: nil)
        let yCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("y")._syntaxNode,
                                                            trailingComma: nil)
        let zCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("z")._syntaxNode,
                                                            trailingComma: nil)
        
        result = result.appending(.init({ (builder) in
            builder.useCondition(xCondition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken(leadingTrivia: .zero,
                                                                  trailingTrivia: .spaces(1)))
        }))
        
        result = result.appending(.init({ (builder) in
            builder.useCondition(yCondition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken(leadingTrivia: .zero,
                                                                  trailingTrivia: .spaces(1)))
        }))
        
        result = result.appending(.init({ (builder) in
            builder.useCondition(zCondition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken(leadingTrivia: .zero))
        }))
        
        XCTAssertEqual(result.description, "x, y, z,")
        
        // when
        result.removeLastTrailingCommaIfNeeded()
        
        // then
        XCTAssertEqual(result.description, "x, y, z")
    }
    
    func test_removeLastTrailingCommaIfNeeded_multipleConditions_trailingCommaNotPresent() {
        // given
        var result = SyntaxFactory.makeBlankConditionElementList()
        let xCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("x")._syntaxNode,
                                                            trailingComma: nil)
        let yCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("y")._syntaxNode,
                                                            trailingComma: nil)
        let zCondition = SyntaxFactory.makeConditionElement(condition: SyntaxFactory.makeIdentifier("z")._syntaxNode,
                                                            trailingComma: nil)
        result = result.appending(.init({ (builder) in
            builder.useCondition(xCondition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken(leadingTrivia: .zero,
                                                                  trailingTrivia: .spaces(1)))
        }))
        
        result = result.appending(.init({ (builder) in
            builder.useCondition(yCondition._syntaxNode)
            builder.useTrailingComma(SyntaxFactory.makeCommaToken(leadingTrivia: .zero,
                                                                  trailingTrivia: .spaces(1)))
        }))
        
        result = result.appending(.init({ (builder) in
            builder.useCondition(zCondition._syntaxNode)
        }))
        
        XCTAssertEqual(result.description, "x, y, z")
        
        // when
        result.removeLastTrailingCommaIfNeeded()
        
        // then
        XCTAssertEqual(result.description, "x, y, z")
    }
}
