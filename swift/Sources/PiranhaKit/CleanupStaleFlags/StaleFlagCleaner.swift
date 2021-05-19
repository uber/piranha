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

class XPFlagCleaner: SyntaxRewriter {
    
    // specifies the value returned by evaluating the expression
    private enum Value {
        case isTrue
        case isFalse
        case isBot
    }

    // specifies the various operators: for now, simply binary
    private enum Operator {
        case and
        case or
        case nilcoalesc
        case unknown
    }

    private let config: PiranhaConfig
    
    private let SELFDOT: String = "self."
    private let UNKNOWN: String = "unknown"

    private var valueMap = [String: Value]()
    private var deepCleanMap = [String: Value]()
    private var fieldMap = [String: Bool]()

    private var previousTrivia: Trivia = []
    private var caseIndex: Int?
    
    private var scopes = [Scope]()

    private let flagName: String
    private let groupName: String
    private let isTreated: Bool

    private var isDeepCleanPass: Bool
    private var shouldDeepClean: Bool

    init(with config: PiranhaConfig,
         flag flagName: String,
         behavior isTreated: Bool,
         group groupName: String) {
        self.config = config
        self.flagName = flagName
        self.isTreated = isTreated
        self.groupName = groupName

        isDeepCleanPass = false
        shouldDeepClean = false
    }

    // checks for the binary operator to handle and/or
    private func findOperator(from node: BinaryOperatorExprSyntax?) -> Operator {
        guard let node = node else {
            return Operator.unknown
        }

        let opTokenKind = node.operatorToken.tokenKind
        if opTokenKind == TokenKind.spacedBinaryOperator("&&") {
            return Operator.and
        } else if opTokenKind == TokenKind.spacedBinaryOperator("||") {
            return Operator.or
        } else if opTokenKind == TokenKind.spacedBinaryOperator("??") {
            return Operator.nilcoalesc
        }
        return Operator.unknown
    }

    // given an argument list, get the argument at a specified index
    private func argument(arglist args: TupleExprElementListSyntax,
                          _ index: Int) -> TupleExprElementSyntax? {
        for (loopindex, argument) in args.enumerated() {
            if loopindex == index {
                return argument
            }
        }
        return nil
    }

    // Helper function to get the type of a node. Used for debugging
    private func type(of node: Syntax) -> Any.Type {
        Mirror(reflecting: node).subjectType
    }

    // Returns the string representation of the node under consideration by concatenating all the tokens
    // helps avoid trivia that is added, if any
    private func string(of node: Syntax) -> String {
        let tokenTexts = node.tokens.map { token in token.text }
        return tokenTexts.joined()
    }

    // matches an argument name with the argument at a specific index to the function call.
    private func match(in node: FunctionCallExprSyntax,
                       name: String, at index: Int) -> Bool {
        if node.argumentList.count > 0,
            let argument = argument(arglist: node.argumentList, index) {
            if let expr = MemberAccessExprSyntax.init(Syntax(argument.expression)) {
                if expr.name.description == name {
                    return true
                }
                if expr.name.description == "asString",
                   expr.dot.previousToken?.description == name {
                    return true
                }
            }
            
            if let expr = ExprSyntax.init(Syntax(argument.expression)),
               labelReplacementAllowed(forIdentifier: expr.description) {
                return true
            }
            if let expr = StringLiteralExprSyntax.init(Syntax(argument.expression)) {
                if name == expr.description.replacingOccurrences(of: "\"", with: "") {
                   return true
               }
            }
        }
        return false
    }
    
    override func visitPre(_ node: Syntax) {
        if let _ = node.asProtocol(DeclSyntaxProtocol.self) as? FunctionDeclSyntax {
            scopes.append(Scope())
        }
        super.visitPre(node)
    }
    
    override func visitPost(_ node: Syntax) {
        if let _ = node.asProtocol(DeclSyntaxProtocol.self) as? FunctionDeclSyntax {
            scopes.removeLast()
        }
        super.visitPost(node)
    }

    // gets the type of the flag API
    private func flagApiType(of node: FunctionCallExprSyntax) -> Method.FlagType? {
        for method in config.methods {
            if node.calledExpression.description.hasSuffix(method.methodName),
                match(in: node, name: flagName, at: method.flagIndex) {
                var foundMatch = true
                // if there is a groupIndex and it doesn't match with groupName,
                if method.groupIndex != nil, !match(in: node, name: groupName, at: method.groupIndex!) {
                    foundMatch = false
                }
                if foundMatch {
                    return method.flagType
                }
            }
        }
        return nil
    }

    // appends the input statements to the result node and returns the updated result node along with information on whether the last node is a return
    private func append(node: CodeBlockItemListSyntax, with stmts: CodeBlockItemListSyntax) -> (CodeBlockItemListSyntax, Bool) {
        var result = node
        var lastNodeIsReturn = false
        for statement in stmts {
            result = result.appending(statement)
            if let _ = ReturnStmtSyntax.init(statement.item) {
                lastNodeIsReturn = true
                break
            }
        }
        return (result, lastNodeIsReturn)
    }

    // get the prefix from node upto the specified index
    // TODO: How to use node.prefix(index) and convert it into ExprListSyntax?
    private func prefix(from node: ExprListSyntax, upto index: Int) -> ExprListSyntax {
        var result = SyntaxFactory.makeBlankExprList()
        for (loopindex, expr) in node.enumerated() {
            if loopindex < index {
                result = result.appending(expr)
            } else {
                break
            }
        }
        return result
    }

    // get the suffix from node starting at index
    // TODO: How to use node.suffix(...) and convert it into ExprListSyntax?
    private func suffix(from node: ExprListSyntax, after index: Int) -> ExprListSyntax {
        var result = SyntaxFactory.makeBlankExprList()
        for (loopindex, expr) in node.enumerated() {
            if loopindex > index {
                result = result.appending(expr)
            }
        }
        return result
    }

    // Returns an element from exprlist at the given index
    private func element(from exprlist: ExprListSyntax, at index: Int) -> ExprSyntax? {
        // TODO: Value of type 'ExprListSyntax' has no subscripts
        // return (index < exprlist.count) ? exprlist[index] : nil
        for (loopindex, expr) in exprlist.enumerated() {
            if loopindex == index {
                return expr
            }
        }
        return nil
    }

    // helper for caching the value into the valueMap.
    private func cache(expression node: Syntax, with value: Value) -> Value {
        valueMap[node.description] = value
        return value
    }

    // evaluates different expressions and is the core algorithm for modifications
    private func evaluate(expression node: Syntax) -> Value {
        // if it is a deep clean pass and the relevant node (e.g., field access) is present in the deepCleanMap,
        // return the associated value.
        if isDeepCleanPass, let value = deepCleanMap[string(of: node)] {
            return value
        }
       
        if let value = scopeResolvedValue(for: node) {
            return value
        }
        // if the expression is previously evaluated in the current pass, return the value
        if let value = valueMap[node.description] {
            return value
        }
        
        if let booleanNode = BooleanLiteralExprSyntax.init(node) {
            // handles "true" or "false"
            if booleanNode.booleanLiteral.tokenKind == TokenKind.trueKeyword {
                return Value.isTrue
            }
            if case .identifier("true") = booleanNode.booleanLiteral.tokenKind {
                return Value.isTrue
            }
            return Value.isFalse
        }
        else if let prefixOperatorNode = PrefixOperatorExprSyntax.init(node) {
            // handles negation
            if prefixOperatorNode.operatorToken?.tokenKind == TokenKind.prefixOperator("!") {
                let value = evaluate(expression: Syntax(prefixOperatorNode.postfixExpression))
                if value == Value.isTrue {
                    return cache(expression: Syntax(prefixOperatorNode), with: Value.isFalse)
                } else if value == Value.isFalse {
                    return cache(expression: Syntax(prefixOperatorNode), with: Value.isTrue)
                }
            }
            return cache(expression: Syntax(prefixOperatorNode), with: Value.isBot)
        } else if let initializerNode = InitializerClauseSyntax.init(node) {
            if let _ = BooleanLiteralExprSyntax.init(Syntax(initializerNode.value)) {
                return Value.isBot
            }
            return evaluate(expression: Syntax(initializerNode.value))
        } else if let conditionElementListNode = ConditionElementListSyntax.init(node) {
            // handles condition element lists
            for element in conditionElementListNode {
                let value = evaluate(expression: element.condition)
                // if there is only one element in the condition, just return its value
                // if there are multiple elements and one of them is false, return that
                if conditionElementListNode.count == 1 || value == Value.isFalse {
                    return cache(expression: Syntax(conditionElementListNode), with: value)
                }
            }
        } else if let functionCallExprNode = FunctionCallExprSyntax.init(node) {
            // handles flag API calls
            var value = Value.isBot
            let api = flagApiType(of: functionCallExprNode)
            if api == .treated  {
                value = (isTreated ? Value.isTrue : Value.isFalse)
            } else if api == .control {
                value = (isTreated ? Value.isFalse : Value.isTrue)
            }
            return cache(expression: Syntax(functionCallExprNode), with: value)
        } else if let sequenceExprNode = SequenceExprSyntax.init(node) {
            // handles sequence expressions
            // examples include:
            // a && b || c && d
            // a = b && c || d
            // a ?? b, etc
            return cache(expression: sequenceExprNode._syntaxNode, with: evaluate(sequence: sequenceExprNode))
        } else if let tupleExprNode = TupleExprSyntax.init(node) {
            // handle (a)
            // TODO: Is there a better way of getting the first element from this collection?
            if let firstChild = tupleExprNode.elementList.first(where: { $0.indexInParent == 0 }) {
                return cache(expression: Syntax(tupleExprNode), with: evaluate(expression: Syntax(firstChild.expression)))
            }
        } else {
            // for all other cases, cache bot val
            return cache(expression: node, with: Value.isBot)
        }
        return Value.isBot
    }
    
    private func scopeResolvedValue(for node: Syntax) -> Value? {
        guard let functionCallExprNode = FunctionCallExprSyntax.init(node) else {
            return nil
        }
        let api = flagApiType(of: functionCallExprNode)
        if api == .treated  {
            return (isTreated ? Value.isTrue : Value.isFalse)
        } else if api == .control {
            return (isTreated ? Value.isFalse : Value.isTrue)
        }
        return nil
    }

    // evaluate a given sequence
    private func evaluate(sequence expr: SequenceExprSyntax) -> Value {
        if expr.elements.count == 1,
            let onlyElement = element(from: expr.elements, at: 0) {
            return evaluate(expression: Syntax(onlyElement))
        }

        for (index, expression) in expr.elements.enumerated() {
            if let _ = AssignmentExprSyntax.init(Syntax(expression)) {
                return Value.isBot
            }
            if let _ = TernaryExprSyntax.init(Syntax(expression)) {
                return Value.isBot
            }

            if let binaryExpr = BinaryOperatorExprSyntax.init(Syntax(expression)) {
                let opKind = findOperator(from: binaryExpr)
                if opKind == Operator.and || opKind == Operator.or {
                    let lhs = prefix(from: expr.elements, upto: index)
                    let rhs = suffix(from: expr.elements, after: index)
                    let result = evaluate(lhs: lhs, rhs: rhs, kind: opKind)
                    if result != Value.isBot {
                        return result
                    }
                } else if opKind == Operator.nilcoalesc {
                    return evaluate(expression: Syntax(element(from: expr.elements, at: 0)!))
                }
            }
        }
        return Value.isBot
    }

    // evaluate a binary expression, given lhs value, rhs value and the operation kind
    private func evaluate(lhs: ExprListSyntax, rhs: ExprListSyntax, kind opKind: Operator) -> Value {
        let lhsVal = evaluate(expression: Syntax(SyntaxFactory.makeSequenceExpr(elements: lhs)))
        let rhsVal = evaluate(expression: Syntax(SyntaxFactory.makeSequenceExpr(elements: rhs)))

        if opKind == Operator.or {
            if lhsVal == Value.isTrue || rhsVal == Value.isTrue {
                return Value.isTrue
            }
            if lhsVal == Value.isFalse, rhsVal == Value.isFalse {
                return Value.isFalse
            }
        } else if opKind == Operator.and {
            if lhsVal == Value.isTrue && rhsVal == Value.isTrue {
                return Value.isTrue
            }
            if lhsVal == Value.isFalse || rhsVal == Value.isFalse {
                return Value.isFalse
            }
        }

        return Value.isBot
    }

    // Evaluate the node of type ClosureExprSyntax
    // In a return/if, if the node contains the flagName + ".asString"/".rawValue",
    // return true.
    // this pattern may be custom usage of flags and may not be generically applicable.
    // If so, can be put behind an option.
    private func evaluate(node: ClosureExprSyntax) -> Bool {
        let key = flagName + ".asString"
        let value = flagName + ".rawValue"

        for statement in node.statements {
            if let returnNode = ReturnStmtSyntax.init(statement.item) {
                if returnNode.description.contains(key) || returnNode.description.contains(value) {
                    return true
                }
            } else if let ifNode = IfStmtSyntax.init(statement.item) {
                if ifNode.description.contains(key) || ifNode.description.contains(value) {
                    return true
                }
            }
        }
        return false
    }

    // simplify an exprlist and return the updated exprlist along with its evaluation
    private func simplify(exprlist: ExprListSyntax) -> (ExprListSyntax, Value) {
        let sequence = SyntaxFactory.makeSequenceExpr(elements: exprlist)

        // evaluate the sequence. if it is a bot, explore possibility of further simplification within itself
        // e.g., a && true is a bot, but can be reduced to a
        let value = evaluate(expression: Syntax(sequence))
        if value == Value.isBot,
            let updated = SequenceExprSyntax.init(Syntax(simplify(node: sequence))) {
            return (updated.elements, value)
        }
        // unable to do any further simplification
        return (exprlist, value)
    }

    // Used for simplifying binary expressions
    // simplify the expression list of elements containing operator of kind opKind at
    // index. always returns a non-empty sequence
    private func simplify(exprlist elements: ExprListSyntax, containing opKind: Operator,
                          at index: Int) -> ExprListSyntax {
        // get the lhs and rhs lists
        var lhs = prefix(from: elements, upto: index)
        var rhs = suffix(from: elements, after: index)

        var lhsVal: Value
        var rhsVal: Value

        (lhs, lhsVal) = simplify(exprlist: lhs)
        (rhs, rhsVal) = simplify(exprlist: rhs)

        // if there are concrete values, one side of the list could be discarded
        if (opKind == Operator.and && lhsVal == Value.isTrue) ||
            (opKind == Operator.or && lhsVal == Value.isFalse) {
            return rhs
        } else if (opKind == Operator.and && rhsVal == Value.isTrue) ||
            (opKind == Operator.or && rhsVal == Value.isFalse) {
            return lhs
        }

        // if neither side could be discarded, append the reduced lists along with the operator and return
        lhs = lhs.appending(element(from: elements, at: index)!)
        for v in rhs {
            lhs = lhs.appending(v)
        }
        return lhs
    }

    // Used for simplifying ternary expressions
    // Simplify the exprlist with ternary node at index.
    // If the ternary node conditional expression is true, pick the first choice.
    // Otherwise, the second choice. If the evaluation returns a false, no simplification
    // is performed.
    private func simplify(exprlist: ExprListSyntax, ternary node: TernaryExprSyntax, at index: Int) -> ExprListSyntax {
        var result = SyntaxFactory.makeBlankExprList()

        // split the expression at index
        // TODO: Explore whether there a better way for doing the split in Swift?
        for (loopindex, expr) in exprlist.enumerated() {
            if loopindex < index {
                result = result.appending(expr)
            } else {
                break
            }
        }

        let v = evaluate(expression: Syntax(node.conditionExpression))
        switch v {
        case Value.isTrue:
            result = result.appending(node.firstChoice)
        case Value.isFalse:
            result = result.appending(node.secondChoice)
        case Value.isBot:
            result = exprlist
        }
        return result
    }
    
    private func updateFieldMap(with key: String) {
        fieldMap[key] = true
        fieldMap[SELFDOT + key] = true
    }
    
    private func updateDeepCleanMap(with key: String, value val: Value) {
        shouldDeepClean = true
        deepCleanMap[key] = val
        deepCleanMap[SELFDOT + key] = val
    }

    // Used for simplifying assignments
    // This function needs to be refactored, especially the DeepCleanPass parts
    private func simplify(assignment exprlist: ExprListSyntax) -> ExprListSyntax {
        let lhs = element(from: exprlist, at: 0)!
        if isDeepCleanPass {
            var key: String?
            if let memberAccessExpr = MemberAccessExprSyntax.init(Syntax(lhs)) {
                key = string(of: Syntax(memberAccessExpr))
            } else if let identifierExpr = IdentifierExprSyntax.init(Syntax(lhs)) {
                key = SELFDOT + string(of: Syntax(identifierExpr))
            }

            if deepCleanMap.keys.contains(key ?? UNKNOWN) {
                // FIXME: currently, refactors even new definitions in the deep clean pass if the
                // variable was simplified in the previous pass.
                return SyntaxFactory.makeBlankExprList()
            }
        }

        var rhs = SyntaxFactory.makeBlankExprList()
        for (loopindex, expr) in exprlist.enumerated() {
            // the rhs of the assignment containing the closureexpression.
            if loopindex > 1 {
                if let closureExpr = ClosureExprSyntax.init(Syntax(expr)),
                    evaluate(node: closureExpr) {
                    // if the closure expression evaluates to true,
                    // (i.e., contains the flag name .asString/.rawValue)
                    // delete the entire assignment and return empty expression list
                    return SyntaxFactory.makeBlankExprList()
                }
                rhs = rhs.appending(expr)
            }
        }

        // handles the scenarios where the RHS is one element and has an API that is not
        // evaluated but is removed (e.g., flag testing API)
        // The resultant refactoring will delete the entire assignment
        if rhs.count == 1 {
            if let node = element(from: exprlist, at: 2),
                let callExpr = FunctionCallExprSyntax.init(Syntax(node)) {
                if flagApiType(of: callExpr) == .testing {
                    return SyntaxFactory.makeBlankExprList()
                } else {
                    let rhsValue = evaluate(expression: Syntax(node))
                  
                    // if it is the initial pass, then update the deepCleanMap because there is
                    // an assignment that evaluated to true or false, and all accesses of that
                    // lhs should also be evaluated appropriately for further cleanup
                    if !isDeepCleanPass, rhsValue != Value.isBot {
                        if let memberAccessExpr = MemberAccessExprSyntax.init(Syntax(lhs)) {
                            shouldDeepClean = true

                            // TODO: Comment on why each key is being put in the deepCleanMap
                            var key = string(of: Syntax(memberAccessExpr))
                            deepCleanMap[key] = rhsValue // put in qualified fieldName

                            key = string(of: Syntax(memberAccessExpr.name))
                            deepCleanMap[key] = rhsValue // put in fieldName
                        } else if let identifierExpr = IdentifierExprSyntax.init(Syntax(lhs)) {
                            let key = string(of: Syntax(identifierExpr))
                            updateDeepCleanMap(with: key, value: rhsValue)
                        }
                    }
                }
            }
        }

        
        if let assignment = element(from: exprlist, at: 1) {
            let rhsSequence = SyntaxFactory.makeSequenceExpr(elements: rhs)
            return SyntaxFactory.makeExprList([lhs, assignment, simplify(node: rhsSequence)])
        }

        // unable to simplify.  so, return as is
        return exprlist
    }

    // helper to handle seq expression
    private func simplify(node: SequenceExprSyntax) -> ExprSyntax {
        let value = evaluate(expression: Syntax(node))
        switch value {
        case Value.isTrue:
            let booleanLiteralExpr = SyntaxFactory.makeBooleanLiteralExpr(booleanLiteral: SyntaxFactory.makeToken(.identifier("true"),
                                                                                                                  presence: .present))
            return ExprSyntax.init(booleanLiteralExpr)
        case Value.isFalse:
            let booleanLiteralExpr = SyntaxFactory.makeBooleanLiteralExpr(booleanLiteral: SyntaxFactory.makeToken(.identifier("false"),
                                                                                                                  presence: .present))
            return ExprSyntax.init(booleanLiteralExpr)
        case Value.isBot:
            var result = SyntaxFactory.makeBlankExprList()
            for (index, expr) in node.elements.enumerated() {
                if InitializerClauseSyntax.init(Syntax(expr)) != nil ||
                AssignmentExprSyntax.init(Syntax(expr)) != nil {
                    result = simplify(assignment: node.elements)
                    if result.count == 0 {
                        return ExprSyntax.init(SyntaxFactory.makeBlankSequenceExpr())
                    }
                } else if let binaryExpr = BinaryOperatorExprSyntax.init(Syntax(expr)) {
                    let opKind = findOperator(from: binaryExpr)
                    result = simplify(exprlist: node.elements, containing: opKind, at: index)
                    return super.visit(node.withElements(result))
                } else if let ternaryExpr = TernaryExprSyntax.init(Syntax(expr)) {
                    result = simplify(exprlist: node.elements, ternary: ternaryExpr, at: index)
                    return super.visit(node.withElements(result))
                }

                if result.count > 0 {
                    return super.visit(node.withElements(result))
                }
            }
        }

        return super.visit(node)
    }

    override func visit(_ node: ConditionElementListSyntax) -> Syntax {
        // if it is just one node, just let the visitor for that node perform the processing
        if node.count == 1 {
            return super.visit(node)
        }

        var result = SyntaxFactory.makeBlankConditionElementList()
        
        for expr in node {
            let value = evaluate(expression: expr.condition)
            if value != Value.isTrue {
                result = result.appending(expr)
            }
        }
        result.removeLastTrailingCommaIfNeeded()
        return super.visit(result)
    }

    override func visit(_ node: SequenceExprSyntax) -> ExprSyntax {
        // handling some custom code that should not be refactored
        if node.description.hasSuffix("recordMode = false || platformUIChange") {
            return super.visit(node)
        }
        return simplify(node: node)
    }

    override func visit(_ node: ArrayElementListSyntax) -> Syntax {
        var newNode = SyntaxFactory.makeBlankArrayElementList()
        for expr in node {
            if !expr.description.contains(flagName) {
                newNode = newNode.appending(expr)
            }
        }
        return super.visit(newNode)
    }
    

   override func visit(_ node: EnumDeclSyntax) -> DeclSyntax {
        if node.identifier.description == flagName {
            return DeclSyntax.init(SyntaxFactory.makeBlankEnumDecl())
        }
        return super.visit(node)
    }

    /*
     ---------------------------
     case random_flag // comment1
     // comment2
     case stale_flag // comment3
     // comment 4
     case another_flag
     ---------------------------

     should translate to

     ----------------------------
     case random_flag // comment1
     //comment4
     case another_flag
     ----------------------------
     */
    override func visit(_ node: EnumCaseDeclSyntax) -> DeclSyntax {
        guard node.elements.count > 0 else {
            return super.visit(node)
        }

        var indexInParent: Int?
        if let nodeparent = node.parent {
            indexInParent = nodeparent.indexInParent
        }

        // TODO: Is there a better way of getting the first element from this sequence?
        if let firstElement = node.elements.first(where: { $0.indexInParent == 0 }),
            flagName == string(of: Syntax(firstElement)),
            let indexInParent = indexInParent {
            caseIndex = indexInParent + 1
            if let leadingTrivia = node.leadingTrivia {
                previousTrivia = []
                for i in leadingTrivia {
                    previousTrivia = previousTrivia.appending(i) // saves comment1
                    if case .newlines = i {
                        break
                    }
                }
            }
            /* this gets rid of the leading trivia for the current decl
             e.g., some comments and case flagname will be handled
             From the above example, gets rid of
              -----------------
              // comment1
              // comment2
              case stale_flag
             */
            return DeclSyntax.init(SyntaxFactory.makeBlankEnumCaseDecl())
        }

        // this will handle the leading trivia for the next token.
        // e.g., case stale_flagname // this flag is stale
        //       case random_flag //
        // the string "// this flag is stale" is attached to the node "case random_flag"
        // so, we need to perform the following operations to cleanup the code
        defer {
            caseIndex = nil
            previousTrivia = []
        }
        if caseIndex == indexInParent,
            let leadingTrivia = node.leadingTrivia {
            var updatedTrivia: Trivia = []

            // update any previous trivia from the deleted node
            // adds "// comment1" to the trivia
            for trivia in previousTrivia {
                updatedTrivia = updatedTrivia.appending(trivia)
            }
            // remove the first trivia piece and leave the rest
            // drops " //comment3" and adds "//comment4"
            for trivia in leadingTrivia.dropFirst() { 
                updatedTrivia = updatedTrivia.appending(trivia)
            }
            /* update the keyword
             the newtrivia will be
             // comment1
             //comment4
             */
            let newCaseKeyword = node.caseKeyword.withLeadingTrivia(updatedTrivia)
            // visit with the updated keyword
            return super.visit(node.withCaseKeyword(newCaseKeyword))
        }
        return super.visit(node)
    }

    override func visit(_ node: MemberDeclBlockSyntax) -> Syntax {
        // update the list of possible fields that is used to filter the deepCleanMap
        if !isDeepCleanPass {
            for member in node.members {
                if let variableDecl = VariableDeclSyntax.init(Syntax(member.decl)) {
                    for binding in variableDecl.bindings {
                        updateFieldMap(with: string(of: Syntax(binding.pattern)))
                    }
                }
            }
        }
        return super.visit(node)
    }

    override func visit(_ node: VariableDeclSyntax) -> DeclSyntax {
        for binding in node.bindings {
            if isDeepCleanPass, deepCleanMap.keys.contains(SELFDOT + string(of: Syntax(binding.pattern))) {
                return visit(SyntaxFactory.makeBlankVariableDecl())
            }

            if let rhs = binding.initializer {
                updateScope(withIdentifier: binding.pattern.description.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines),
                            havingValue: false)
                let rhsValue = evaluate(expression: Syntax(rhs))
                if rhsValue != Value.isBot {
                    let key = string(of: Syntax(binding.pattern))
                    updateFieldMap(with: key)
                    updateDeepCleanMap(with: key, value: rhsValue)
                    return visit(SyntaxFactory.makeBlankVariableDecl())
                }
                if let value = FunctionCallExprSyntax.init(Syntax(rhs.value)) {
                    if (flagApiType(of: value) == .testing) || (evaluate(expression: Syntax(rhs.value)) != Value.isBot) {
                        return visit(SyntaxFactory.makeBlankVariableDecl())
                    }
                }
                if let value = MemberAccessExprSyntax.init(Syntax(rhs.value)),
                    value.description.hasSuffix(flagName) {
                    updateScope(withIdentifier: binding.pattern.description.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines),
                                havingValue: true)
                    return visit(SyntaxFactory.makeBlankVariableDecl())
                }
            }
        }
        return super.visit(node)
    }

    
    var replacements: [BooleanLiteralExprSyntax] = []
    override func visit(_ node: FunctionCallExprSyntax) -> ExprSyntax {
        let value = evaluate(expression: Syntax(node))
        switch value {
        case Value.isTrue:
            let booleanLiteralExpr = SyntaxFactory.makeBooleanLiteralExpr(booleanLiteral: SyntaxFactory.makeToken(.identifier("true"),
                                                                                                                  presence: .present))
            return visit(booleanLiteralExpr)
        case Value.isFalse:
            let booleanLiteralExpr = SyntaxFactory.makeBooleanLiteralExpr(booleanLiteral: SyntaxFactory.makeToken(.identifier("false"),
                                                                                                                  presence: .present))
            return visit(booleanLiteralExpr)
        case Value.isBot:
            return super.visit(node)
        }
    }

    override func visit(_ node: CodeBlockItemListSyntax) -> Syntax {
        var newBody = SyntaxFactory.makeBlankCodeBlockItemList()
        var lastNodeIsReturn = false

        for statement in node {
            if let ifNode = IfStmtSyntax.init(statement.item) {
                let value = evaluate(expression: Syntax(ifNode.conditions))
                switch value {
                case Value.isBot: newBody = newBody.appending(statement)
                case Value.isTrue:
                    (newBody, lastNodeIsReturn) = append(node: newBody, with: ifNode.body.statements)
                case Value.isFalse:
                    if let elseBodyNode = ifNode.elseBody,
                       let elseBody = CodeBlockSyntax.init(Syntax(elseBodyNode)) {
                        (newBody, lastNodeIsReturn) = append(node: newBody, with: elseBody.statements)
                    }
                }
                if lastNodeIsReturn {
                    return super.visit(newBody)
                }
            } else if let guardNode = GuardStmtSyntax.init(statement.item) {
                let value = evaluate(expression: Syntax(guardNode.conditions))
                switch value {
                case Value.isBot: newBody = newBody.appending(statement)
                case Value.isTrue:
                    break
                case Value.isFalse:
                    (newBody, lastNodeIsReturn) = append(node: newBody, with: guardNode.body.statements)
                    if lastNodeIsReturn {
                        return super.visit(newBody)
                    }
                }
            } else if let callNode = FunctionCallExprSyntax.init(statement.item),
                      flagApiType(of: callNode) == .testing {
                // do nothing, as the test API needs to be discarded
            } else {
                newBody = newBody.appending(statement)
            }
        }
        return super.visit(newBody)
    }

    func setNextPass() {
        // clear up all cached valuemaps.
        valueMap.removeAll()

        // if it is not a field, remove it from deepcleaning
        // update the valuemap also.
        for (key, value) in deepCleanMap {
            if !fieldMap.keys.contains(key) {
                deepCleanMap.removeValue(forKey: key)
            } else {
                valueMap[key] = value
            }
        }

        if shouldDeepClean {
            isDeepCleanPass = true
        }
    }

    func deepClean() -> Bool {
        return shouldDeepClean
    }
}

private extension XPFlagCleaner {
    
    
    final class Scope {
        // True signifies that the replacement can happen and false signifies that replacement shouldn't happen.
        var labelVariableMap: [String: Bool] = [:]
    }
    
    
    private func updateScope(withIdentifier identifier: String,
                             havingValue value: Bool) {
        guard let scope = scopes.last else { return }
        scope.labelVariableMap[identifier] = value
    }
    
    private func labelReplacementAllowed(forIdentifier identifier: String) -> Bool {
        for scope in scopes.reversed() {
            /// If the closest scope label is found that allows replacement, return true.
            if scope.labelVariableMap[identifier] == true {
                return true
            }
            /// If the closest scope label is found that doesn't allow replacement, return false.
            if scope.labelVariableMap[identifier] == false {
                return false
            }
        }
        return false
    }
}
