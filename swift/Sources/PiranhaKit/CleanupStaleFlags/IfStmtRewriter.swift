//
//  File.swift
//  
//
//  Created by Chirag Ramani on 13/05/21.
//

import Foundation
import SwiftSyntax

/// This SyntaxRewriter dedicatedly operates on the if-else condition and also works on the if-else ladder.
/// It does the following:
/// a.) to remove the conditional block that has the false condition because that will never be executed.
/// b.) If there is a conditional block evaluating to true, then it removes all the following conditions as well as their respective bodies because they will never be executed.
final class IfElseStmtRewriter: SyntaxRewriter {
    
    override func visit(_ node: IfStmtSyntax) -> StmtSyntax {
        return super.visit(reduce(node))
    }
    
    // MARK: Private
    
    private func reduce(_ node: IfStmtSyntax) -> IfStmtSyntax {
        var updatedNode = reduceFalseTreeIfApplicable(node)
        updatedNode = reduceTrueTreeIfApplicable(updatedNode)
        return updatedNode
    }
    
    private func reduceFalseTreeIfApplicable(_ node: IfStmtSyntax) -> IfStmtSyntax {
        guard node.conditions.count == 1,
              node.conditions.first?.description.trimmingCharacters(in: .whitespacesAndNewlines) == "false" else {
            /// Since this doesn't meet the false cleanup criteria, therefore returning.
            return node
        }
        /// Processing else block if present.
        if let _ = node.elseKeyword,
           let elseBody = node.elseBody {
            // Is it an only else condition and there is no if condition.
            if let expr = CodeBlockSyntax.init(elseBody) {
                var ifStmt = SyntaxFactory.makeBlankIfStmt()
                ifStmt = ifStmt.withBody(codeBlockFor(node: node,
                                                      referenceCodeBlock: expr))
                return ifStmt
            }
            
            // If condition is present.
            if let ifOfBody = IfStmtSyntax.init(elseBody) {
                var ifStmt = SyntaxFactory.makeBlankIfStmt()
                ifStmt = reduce(ifOfBody)
                return ifStmt
            }
            
            return node
        } else {
            // False conditional and no else body, hence returning a blank statement.
            return SyntaxFactory.makeBlankIfStmt()
        }
    }
    
    /// This update the code block with the appropriate leading and trailing trivia.
    private func codeBlockFor(node: IfStmtSyntax,
                              referenceCodeBlock codeBlock: CodeBlockSyntax,
                              previousNode: IfStmtSyntax? = nil) -> CodeBlockSyntax {
        if node.ifKeyword.previousToken?.tokenKind == .elseKeyword {
            /// Since there is an existing else block, we would want to have braces and their respective trivia.
            let leading = SyntaxFactory.makeToken(.leftBrace,
                                                  presence: .present,
                                                  leadingTrivia: Trivia.init(pieces: []),
                                                  trailingTrivia: Trivia.init(pieces: []))
            
            let trailing = SyntaxFactory.makeToken(.rightBrace,
                                                   presence: .present,
                                                   leadingTrivia: codeBlock.rightBrace.leadingTrivia,
                                                   trailingTrivia: codeBlock.rightBrace.trailingTrivia)
            return SyntaxFactory.makeCodeBlock(leftBrace: leading,
                                               statements: codeBlock.statements,
                                               rightBrace: trailing)
        } else {
            /// Matching the trivia to the if keyword.
            var statements = codeBlock.statements
            let firstModified = statements.first?.withLeadingTrivia(node.ifKeyword.leadingTrivia)
            statements = statements.replacing(childAt: 0, with: firstModified!)
            let leading = SyntaxFactory.makeToken(.identifier(""),
                                                  presence: .present,
                                                  leadingTrivia: Trivia.init(pieces: []),
                                                  trailingTrivia: Trivia.init(pieces: []))
            
            let trailing = SyntaxFactory.makeToken(.identifier(""),
                                                   presence: .present,
                                                   leadingTrivia: Trivia.init(pieces: []),
                                                   trailingTrivia: codeBlock.rightBrace.trailingTrivia)
            return SyntaxFactory.makeCodeBlock(leftBrace: leading,
                                               statements: statements,
                                               rightBrace: trailing)
        }
    }
    
    // If the condition is true, no other if else condition(s) below this node  will be executed hence the following blocks will be cleaned-up.
    private func reduceTrueTreeIfApplicable(_ node: IfStmtSyntax) -> IfStmtSyntax {
        guard node.conditions.count == 1,
              node.conditions.first?.description.trimmingCharacters(in: .whitespacesAndNewlines) == "true" else {
            /// Since this doesn't meet the true cleanup criteria, therefore returning.
            return node
        }
        var statement = SyntaxFactory.makeBlankIfStmt()
        statement = statement.withBody(codeBlockFor(node: node,
                                              referenceCodeBlock: node.body))
        return statement
    }
}
