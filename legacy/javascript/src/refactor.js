/**
 * Copyright (c) 2019 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * This module contains Piranha's refactoring logic.
 *
 * Public API -
 *
 * RefactorEngine -
 * Contains methods to carry out different steps of the refactoring process.
 * Also has state variables and parameters shared across these methods.
 *
 * Exposed methods
 *
 * constructor
 * @param {Object} ast - abstract syntax tree parsed from code
 * @param {Object} properties - contains information about flag APIs and flag types parsed from a config file
 * @param {Boolean} behaviour - specifies whether flag is treated or not
 * @param {String} flagname - name of stale feature flag
 * @param {number} max_cleanup_steps - the max number of times deep cleaning must be done
 * @param {Boolean} print_to_console - should a message be printed after refactoring is done, default is false
 * @param {Boolean} keep_comments - should all comments be retained, default is false
 * @param {String} filename - the name of the source file, this argument is optional and only needed to print a helpful console message
 *
 * refactorPipeline
 * Carries out the refactoring process and prints a helpful message at the end.
 *
 * logger
 * A winston logger that can be configured to produce debugging output
 */

const babel = require('@babel/core'); // Convenient API for AST traversal
const winston = require('winston'); // logger
const colors = require('colors');

class RefactorEngine {
    constructor(
        ast,
        properties,
        behaviour,
        flagname,
        max_cleanup_steps,
        print_to_console,
        keep_comments = false,
        filename = null,
    ) {
        this.ast = ast;
        this.properties = properties;
        this.flagname = flagname;
        this.max_cleanup_steps = max_cleanup_steps;
        this.print_to_console = print_to_console;
        this.keep_comments = keep_comments;
        this.behaviour = behaviour;
        this.changed = false;
        this.filename = filename;
    }

    hasFlagKeywordInFile() {
        let flagname = this.flagname;
        let hasFlagKeywordInFile = false;
        babel.traverse(this.ast, {
            enter: function (path) {
                if (
                    (path.node.type === 'StringLiteral' && path.node.value === flagname) ||
                    (path.node.type === 'Identifier' && path.node.name === flagname)
                ) {
                    hasFlagKeywordInFile = true;
                    path.stop();
                }
            },
        });
        return hasFlagKeywordInFile;
    }

    trueLiteral() {
        const vanillaLiteral = {
            // A Boolean Literal is parsed like this by recast
            type: 'BooleanLiteral',
            value: true,
            raw: 'true',
        };

        // This additional property marks that this literal was introduced by Piranha
        vanillaLiteral.createdByPiranha = true;

        return vanillaLiteral;
    }

    falseLiteral() {
        const vanillaLiteral = {
            // A Boolean Literal is parsed like this by recast
            type: 'BooleanLiteral',
            value: false,
            raw: 'false',
        };

        // This additional property marks that this literal was introduced by Piranha
        vanillaLiteral.createdByPiranha = true;

        return vanillaLiteral;
    }

    getMethodHashMap(properties) {
        var methodHashMap = new Map();

        properties['methodProperties'].forEach((item) => {
            methodHashMap.set(item['methodName'], item);
        });

        return methodHashMap;
    }

    // Verify this is a literal introduced by Piranha.
    // We only refactor code associated with literals having this property and leave original code untouched
    isPiranhaLiteral(node) {
        return node.type === 'BooleanLiteral' && node.createdByPiranha !== undefined;
    }

    reduceLogicalExpression(literal, expression, operator) {
        if (operator === '&&') {
            if (literal.value === true) {
                return expression;
            }

            if (literal.value === false) {
                return this.falseLiteral();
            }
        } else if (operator === '||') {
            if (literal.value === true) {
                return this.trueLiteral();
            }

            if (literal.value === false) {
                return expression;
            }
        }
    }

    // Given a candidate function, check if it is redundant, i.e returns a piranhaLiteral
    // The argument `func` refers to a function with a single return statement at the end of its body
    checkAndAddRedundantFunction(func, name, redundantFunctions) {
        if (func.body === null || func.body.body == null) {
            return false;
        }

        const returnIndex = func.body.body.length - 1;

        if (returnIndex < 0)
            // If returnIndex == -1 when function body is empty
            return false;
        else {
            var returnNode = func.body.body[returnIndex];

            if (
                returnNode.type === 'ReturnStatement' &&
                returnNode.argument !== null &&
                this.isPiranhaLiteral(returnNode.argument) &&
                typeof returnNode.argument.value === 'boolean'
            ) {
                redundantFunctions[name] = returnNode.argument.value;
            }

            // TODO introduce a warning here since the assumption of return statement being at the end is violated

            return true;
        }
    }

    // Separate leading, trailing and remaining comments into three lists
    collateCommentsByPosition(node) {
        var collatedComments = [
            [], // leading
            [], // trailing
            [], // remaining
        ];

        if (node.comments == null) {
            return collatedComments;
        }

        return node.comments.reduce(([leadingList, trailingList, remainingList], comment) => {
            if (comment.leading) {
                leadingList.push(comment);
            } else if (comment.trailing) {
                trailingList.push(comment);
            } else {
                remainingList.push(comment);
            }

            return [leadingList, trailingList, remainingList];
        }, collatedComments);
    }

    attachCommentsAtBeginning(node, newComments) {
        if (!Array.isArray(newComments) || newComments.length === 0) {
            return;
        }

        if (node.comments == null) {
            node.comments = [];
        }

        newComments.push(...node.comments);
        node.comments = [...newComments];
    }

    attachCommentsAtEnd(node, newComments) {
        if (!Array.isArray(newComments) || newComments.length === 0) {
            return;
        }

        if (node.comments == null) {
            node.comments = [];
        }

        node.comments.push(...newComments);
    }

    flipCommentPosition(comment) {
        if (comment.leading || comment.trailing) {
            comment.leading = !comment.leading;
            comment.trailing = !comment.trailing;
        }
        return comment;
    }

    moveAllCommentsToSiblings(node, parent) {
        if ('body' in parent && Array.isArray(parent.body)) {
            var nodeIndex = parent.body.indexOf(node);
            var previousSiblingIndex = nodeIndex !== 0 ? nodeIndex - 1 : null;
            var nextSiblingIndex = nodeIndex !== parent.body.length - 1 ? nodeIndex + 1 : null;

            let leadingComments, trailingComments;
            [leadingComments, trailingComments] = this.collateCommentsByPosition(node);

            if (previousSiblingIndex != null) {
                if (nodeIndex === parent.body.length - 1) {
                    let consolidatedComments = leadingComments.map(this.flipCommentPosition);
                    consolidatedComments = consolidatedComments.concat(trailingComments);

                    this.attachCommentsAtEnd(parent.body[previousSiblingIndex], consolidatedComments);
                } else {
                    let consolidatedComments = leadingComments.map(this.flipCommentPosition);

                    this.attachCommentsAtEnd(parent.body[previousSiblingIndex], consolidatedComments);
                }
            }

            if (nextSiblingIndex != null) {
                if (nodeIndex === 0) {
                    let consolidatedComments = leadingComments;
                    consolidatedComments = consolidatedComments.concat(trailingComments.map(this.flipCommentPosition));

                    this.attachCommentsAtBeginning(parent.body[nextSiblingIndex], consolidatedComments);
                } else {
                    let consolidatedComments = trailingComments.map(this.flipCommentPosition);

                    this.attachCommentsAtBeginning(parent.body[nextSiblingIndex], consolidatedComments);
                }
            }
        }
    }

    moveLeadingCommentsToSibling(node, parent) {
        if ('body' in parent && Array.isArray(parent.body)) {
            var nodeIndex = parent.body.indexOf(node);
            var previousSiblingIndex = nodeIndex !== 0 ? nodeIndex - 1 : null;
            var nextSiblingIndex = nodeIndex !== parent.body.length - 1 ? nodeIndex + 1 : null;

            let [leadingComments] = this.collateCommentsByPosition(node);

            if (nextSiblingIndex != null) {
                this.attachCommentsAtBeginning(parent.body[nextSiblingIndex], leadingComments);
            } else if (previousSiblingIndex != null) {
                var consolidatedComments = leadingComments.map(this.flipCommentPosition);

                this.attachCommentsAtEnd(parent.body[previousSiblingIndex], consolidatedComments);
            }
        }
    }

    moveCommentsToConsequent(node) {
        let leadingComments, trailingComments;
        [leadingComments, trailingComments] = this.collateCommentsByPosition(node);

        this.attachCommentsAtBeginning(node.consequent, leadingComments);
        this.attachCommentsAtEnd(node.consequent, trailingComments);
    }

    moveCommentsToAlternate(node) {
        let leadingComments, trailingComments;
        [leadingComments, trailingComments] = this.collateCommentsByPosition(node);

        this.attachCommentsAtBeginning(node.alternate, leadingComments);
        this.attachCommentsAtEnd(node.alternate, trailingComments);
    }

    moveCommentsToExtremeChildren(node, parent, keep_comments) {
        let leadingComments, trailingComments;
        [leadingComments, trailingComments] = this.collateCommentsByPosition(node);

        if (node.body.length !== 0) {
            this.attachCommentsAtBeginning(node.body[0], leadingComments);
            this.attachCommentsAtEnd(node.body[node.body.length - 1], trailingComments);
        } else if (keep_comments) {
            this.attachCommentsAtEnd(parent, node.comments);
        }
    }

    preserveCommentsBasedOnOption(node, parent, keep_comments) {
        if (keep_comments) {
            this.moveAllCommentsToSiblings(node, parent);
        } else {
            this.moveLeadingCommentsToSibling(node, parent);
        }
    }

    classPropertyToLiteral(redundantFunctions) {
        var engine = this;

        babel.traverse(this.ast, {
            enter: function (path) {
                const { node } = path;
                if (node.type === 'MemberExpression') {
                    if (redundantFunctions.hasOwnProperty(node.property.name)) {
                        engine.changed = true;
                        path.replaceWith(engine.trueLiteral());
                    }
                }
            },
        });
    }

    // Replace flag checks with Boolean literals
    // testFlagFunction(flag) -> | true,  if flagType = treated and piranha.treatment = true
    //                           | false,  if flagType = treated and piranha.treatment = false
    //                           | false, if flagType = control and piranha.control = true
    //                           | true, if flagType = control and piranha.control = false
    flagAPIToLiteral() {
        var methodHashMap = this.getMethodHashMap(this.properties);
        var engine = this;
        babel.traverse(this.ast, {
            enter: function (path) {
                const node = path.node;
                if (node.type === 'CallExpression') {
                    let calleeName = node.callee.name;
                    if (node.callee.type === 'MemberExpression') {
                        calleeName = node.callee.property.name;
                    }
                    if (methodHashMap.has(calleeName)) {
                        const argumentIndex = methodHashMap.get(calleeName).argumentIndex;
                        const nodeArgument = node.arguments[argumentIndex];

                        let nodeArgumentIsFlag = false;
                        switch (nodeArgument.type) {
                            case 'Identifier':
                                nodeArgumentIsFlag = nodeArgument.name === engine.flagname;
                                break;
                            case 'StringLiteral':
                                nodeArgumentIsFlag = nodeArgument.value === engine.flagname;
                                break;
                        }
                        if (nodeArgumentIsFlag) {
                            const flagType = methodHashMap.get(calleeName).flagType;
                            engine.changed = true;

                            if (
                                (flagType === 'treated' && engine.behaviour) ||
                                (flagType === 'control' && !engine.behaviour)
                            ) {
                                path.replaceWith(engine.trueLiteral());
                            } else {
                                path.replaceWith(engine.falseLiteral());
                            }
                        }
                    }
                }
            },
        });
    }

    // After converting to boolean literals, do partial evaluation as follows
    //  true AND X -> X, false AND X -> false
    //  true OR X -> true, false OR X -> X
    //  NOT true -> false, NOT false -> true
    evalBoolExpressions() {
        var engine = this;

        babel.traverse(this.ast, {
            exit: function (path) {
                const node = path.node;
                if (node.type === 'LogicalExpression') {
                    var expression1 = node.left;
                    var expression2 = node.right;

                    if (engine.isPiranhaLiteral(expression1)) {
                        engine.changed = true;
                        path.replaceWith(engine.reduceLogicalExpression(expression1, expression2, node.operator));
                    }

                    if (engine.isPiranhaLiteral(expression2)) {
                        engine.changed = true;
                        path.replaceWith(engine.reduceLogicalExpression(expression2, expression1, node.operator));
                    }
                } else if (
                    node.type === 'UnaryExpression' &&
                    node.operator === '!' &&
                    engine.isPiranhaLiteral(node.argument)
                ) {
                    if (node.argument.value === true) {
                        engine.changed = true;
                        path.replaceWith(engine.falseLiteral());
                    } else if (node.argument.value === false) {
                        engine.changed = true;
                        path.replaceWith(engine.trueLiteral());
                    }
                } else if (
                    node.type === 'CallExpression' &&
                    node.callee.name === 'Boolean' &&
                    engine.isPiranhaLiteral(node.arguments[0])
                ) {
                    path.replaceWith(node.arguments[0]);
                }
            },
        });
    }

    // After simplifying boolean expressions reduce if statements if possible
    // if (true) b1 else b2 -> b1
    // if (false) b1 else b2 -> b2
    reduceIfStatements() {
        var engine = this;

        babel.traverse(this.ast, {
            exit: function (path) {
                const node = path.node;
                if (
                    (node.type === 'IfStatement' || node.type === 'ConditionalExpression') &&
                    engine.isPiranhaLiteral(node.test)
                ) {
                    if (node.test.value === true) {
                        // node.consequent is always non-null so no check required
                        engine.changed = true;
                        engine.moveCommentsToConsequent(node);

                        path.replaceWith(node.consequent);
                    } else if (node.test.value === false) {
                        if (node.alternate == null) {
                            engine.changed = true;
                            path.remove();
                        } else {
                            engine.changed = true;
                            engine.moveCommentsToAlternate(node);

                            path.replaceWith(node.alternate);
                        }
                    }
                }
            },
        });

        // Flatten any nested blocks introduced in the previous step by moving their contents to their parent
        babel.traverse(this.ast, {
            exit: function (path) {
                const { node, parent } = path;
                if (node.type === 'BlockStatement' && (parent.type === 'BlockStatement' || parent.type === 'Program')) {
                    engine.moveCommentsToExtremeChildren(node, parent, engine.keep_comments);
                    var nodeIndex = parent.body.indexOf(node);
                    parent.body.splice(nodeIndex, 1, ...node.body);
                }
            },
        });
    }

    // Get variable names which are assigned to literals in the previous steps
    // foo = cond -> _
    // var foo = cond -> _
    // where cond evaluates to a Boolean literal in a previous step
    getRedundantVarnames() {
        var assignments = {};
        var engine = this;

        // Get a list of variable names which are assigned to a boolean literal
        babel.traverse(this.ast, {
            enter: function (path) {
                const node = path.node;
                if (node.type === 'ClassProperty' && node.value) {
                    if (engine.isPiranhaLiteral(node.value) && typeof node.value.value === 'boolean') {
                        assignments[node.key.name] = node.value.value;
                    }
                } else if (node.type === 'VariableDeclaration') {
                    for (var i = 0; i < node.declarations.length; i++) {
                        const declaration = node.declarations[i];
                        if (
                            declaration.init &&
                            engine.isPiranhaLiteral(declaration.init) &&
                            typeof declaration.init.value === 'boolean'
                        ) {
                            assignments[declaration.id.name] = declaration.init.value;
                        }
                    }
                } else if (node.type === 'AssignmentExpression') {
                    if (node.right && engine.isPiranhaLiteral(node.right) && typeof node.right.value === 'boolean') {
                        if (node.left.name !== undefined) assignments[node.left.name] = node.right.value;
                    }
                }
            },
        });

        return assignments;
    }

    // Remove all variable declarations corresponding variables in `assignments`
    // Replace all references of variables in `assignments` with the corresponding literal
    pruneVarReferences(assignments, templateCleanupInfo) {
        var engine = this;

        // Remove redundant variables by deleting declarations and replacing variable references
        babel.traverse(this.ast, {
            enter: function (path) {
                const { node, parent } = path;
                if (node.type === 'VariableDeclaration') {
                    node.declarations.forEach(function (declaration, index) {
                        if (assignments.hasOwnProperty(declaration.id.name)) {
                            engine.changed = true;
                            node.declarations.splice(index, 1);
                        }
                    });
                } else if (node.type === 'ExpressionStatement' && node.expression.type === 'AssignmentExpression') {
                    if (assignments.hasOwnProperty(node.expression.left.name)) {
                        engine.preserveCommentsBasedOnOption(node, parent, engine.keep_comments);
                        engine.changed = true;
                        path.remove();
                    }
                } else if (node.type === 'ClassProperty') {
                    if (assignments.hasOwnProperty(node.key.name)) {
                        engine.changed = true;
                        path.remove();
                        templateCleanupInfo.properties.push(node.key.name);
                    }
                } else if (node.type === 'MemberExpression') {
                    if (path.toString() === `this.${node.property.name}`) {
                        if (assignments[node.property.name] === true) {
                            engine.changed = true;
                            path.replaceWith(engine.trueLiteral());
                        } else if (assignments[node.property.name] === false) {
                            engine.changed = true;
                            path.replaceWith(engine.falseLiteral());
                        }
                    }
                } else if (node.type === 'Identifier') {
                    if (assignments.hasOwnProperty(node.name)) {
                        if (assignments[node.name] === true) {
                            engine.changed = true;
                            path.replaceWith(engine.trueLiteral());
                        } else if (assignments[node.name] === false) {
                            engine.changed = true;
                            path.replaceWith(engine.falseLiteral());
                        }
                    }
                }
            },

            // After previous step, some declaration may have no declarators, delete them.
            exit: function (path) {
                const { node, parent } = path;
                if (node.type === 'VariableDeclaration') {
                    if (node.declarations.length === 0) {
                        engine.preserveCommentsBasedOnOption(node, parent, true);
                        engine.changed = true;
                        return path.remove();
                    }
                }
            },
        });
    }

    // ========= Step-3c ==========
    // Find all functions which have a single return statement
    getFunctionsWithSingleReturn() {
        var numReturns = {};
        var singleReturn = {};
        var current;

        // Create a table mapping function names to number of return statements in them
        babel.traverse(this.ast, {
            enter: function (path) {
                const { node, parent } = path;
                if (node.type === 'ClassMethod') {
                    if (node.kind === 'get') {
                        current = node.key.name;
                        numReturns[current] = 0;
                    } else {
                        current = null;
                    }
                } else if (node.type === 'FunctionDeclaration' && node.id) {
                    current = node.id.name;
                    numReturns[current] = 0;
                } else if (node.type === 'FunctionExpression') {
                    if (parent.type === 'VariableDeclarator') {
                        current = parent.id.name;
                        numReturns[current] = 0;
                    } else {
                        current = null;
                    }
                } else if (node.type === 'ArrowFunctionExpression') {
                    if (parent.type === 'VariableDeclarator') {
                        current = parent.id.name;

                        if (node.body.type !== 'BlockStatement') {
                            numReturns[current] = 1;
                        } else {
                            numReturns[current] = 0;
                        }
                    } else {
                        current = null;
                    }
                } else if (node.type === 'ReturnStatement') {
                    if (current) numReturns[current]++;
                }
            },
        });

        // Filter the map for functions having only one return.
        for (var fun in numReturns) {
            if (numReturns[fun] === 1) {
                singleReturn[fun] = 1;
            }
        }

        return singleReturn;
    }

    // Given a list of functions with single return statements at the end,
    // find all functions which return a Boolean literal introduced in a previous step
    getRedundantFunctions(singleReturnFunctions) {
        var redundantFunctions = {};
        var engine = this;

        babel.traverse(this.ast, {
            enter: function (path) {
                const { node, parent } = path;
                if (node.type === 'ClassMethod' && singleReturnFunctions.hasOwnProperty(node.key.name)) {
                    if (!engine.checkAndAddRedundantFunction(node, node.key.name, redundantFunctions)) {
                        path.skip();
                    }
                } else if (node.type === 'FunctionDeclaration' && node.id && singleReturnFunctions[node.id.name]) {
                    if (!engine.checkAndAddRedundantFunction(node, node.id.name, redundantFunctions)) {
                        path.skip();
                    }
                } else if (node.type === 'FunctionExpression') {
                    if (parent.type === 'VariableDeclarator' && singleReturnFunctions[parent.id.name]) {
                        if (!engine.checkAndAddRedundantFunction(node, parent.id.name, redundantFunctions)) {
                            path.skip();
                        }
                    }
                } else if (node.type === 'ArrowFunctionExpression') {
                    if (parent.type === 'VariableDeclarator' && singleReturnFunctions[parent.id.name] !== undefined) {
                        if (node.body.type !== 'BlockStatement') {
                            if (engine.isPiranhaLiteral(node.body) && typeof node.body.value === 'boolean') {
                                redundantFunctions[parent.id.name] = node.body.value;
                            }
                        } else {
                            if (!engine.checkAndAddRedundantFunction(node, parent.id.name, redundantFunctions)) {
                                path.skip();
                            }
                        }
                    }
                }
            },
        });

        return redundantFunctions;
    }

    // Given a list of functions to be replaced, remove all function declarations
    // replace all calls to the functions with the corresponding literals
    pruneFuncReferences(pruneList, templateCleanupInfo) {
        var engine = this;

        babel.traverse(this.ast, {
            enter: function (path) {
                const { node, parent } = path;
                if (node.type === 'ClassMethod' && pruneList.hasOwnProperty(node.key.name)) {
                    if (engine.keep_comments) {
                        engine.moveAllCommentsToSiblings(node, parent);
                    }

                    engine.changed = true;
                    path.remove();
                    templateCleanupInfo.properties.push(node.key.name);
                }
                if (node.type === 'FunctionDeclaration' && node.id && pruneList.hasOwnProperty(node.id.name)) {
                    if (engine.keep_comments) {
                        engine.moveAllCommentsToSiblings(node, parent);
                    }

                    engine.changed = true;
                    path.remove();
                } else if (node.type === 'VariableDeclarator' && pruneList.hasOwnProperty(node.id.name)) {
                    engine.changed = true;
                    path.remove();
                } else if (node.type === 'CallExpression' && pruneList.hasOwnProperty(node.callee.name)) {
                    if (pruneList[node.callee.name]) {
                        engine.changed = true;
                        path.replaceWith(engine.trueLiteral());
                    } else {
                        engine.changed = true;
                        path.replaceWith(engine.falseLiteral());
                    }
                }
            },

            exit: function (path) {
                const { node, parent } = path;
                if (node.type === 'VariableDeclaration' && node.declarations.length === 0) {
                    engine.preserveCommentsBasedOnOption(node, parent, engine.keep_comments);
                    engine.changed = true;
                    path.remove();
                }
            },
        });

        return engine.changed;
    }

    consolidateReturnStatements() {
        babel.traverse(this.ast, {
            BlockStatement: function (path) {
                const { node } = path;
                const indexOfFirstReturn = node.body.findIndex((elem) => elem.type === 'ReturnStatement');
                if(indexOfFirstReturn !== -1) {
                    node.body.splice(indexOfFirstReturn + 1);   
                }
            },
        });
    }

    cleanupEmptyConstructor() {
        babel.traverse(this.ast, {
            ClassMethod: function (path) {
                const { node } = path;
                if (
                    node.kind === 'constructor' &&
                    (node.body.body.length === 0 ||
                        (node.body.body.length === 1 &&
                            node.body.body[0].type === 'ExpressionStatement' &&
                            node.body.body[0].expression.callee &&
                            node.body.body[0].expression.callee.type === 'Super'))
                ) {
                    path.remove();
                }
            },
        });
    }

    emberSpecificCleanup() {
        const engine = this;
        babel.traverse(this.ast, {
            //computed function cleanup
            CallExpression: function (path) {
                const { node } = path;
                if (node.callee.name === 'computed') {
                    const funcExpression = node.arguments.find((elem) => elem.type === 'FunctionExpression');
                    if (
                        funcExpression &&
                        funcExpression.body.body.length === 1 &&
                        funcExpression.body.body[0].type === 'ReturnStatement' &&
                        funcExpression.body.body[0].argument &&
                        engine.isPiranhaLiteral(funcExpression.body.body[0].argument)
                    ) {
                        path.replaceWith(funcExpression.body.body[0].argument);
                    }
                }
            },
            //Init cleanup
            ClassMethod: function (path) {
                const { node } = path;
                if (
                    node.key.name === 'init' &&
                    !node.decorators &&
                    (node.body.body.length === 0 ||
                        (node.body.body.length === 1 &&
                            node.body.body[0].type === 'ExpressionStatement' &&
                            (path.get('body.body.0').toString().startsWith('super.init') ||
                                path.get('body.body.0').toString().startsWith('this._super'))))
                ) {
                    path.remove();
                }
            },
        });
    }

    // Remove the piranha property from literals so code generator can work properly
    finalizeLiterals() {
        var engine = this;

        babel.traverse(this.ast, {
            enter: function ({ node }) {
                if (engine.isPiranhaLiteral(node)) {
                    delete node.createdByPiranha;
                }
            },
        });
    }

    // Calls the entire pipeline
    refactorPipeline() {
        const hasFlagKeywordInFile = this.hasFlagKeywordInFile();
        let hasAstChanges = false;

        this.flagAPIToLiteral();
        if (this.changed) hasAstChanges = true;

        let iterations = 0;
        this.changed = true;

        let templateCleanupInfo = { properties: [] },
            redundantFunctions = {};
        while (this.changed && iterations < this.max_cleanup_steps) {
            this.changed = false;

            this.classPropertyToLiteral(redundantFunctions);
            this.evalBoolExpressions();
            this.reduceIfStatements();

            var redundantVarnames = this.getRedundantVarnames();
            this.pruneVarReferences(redundantVarnames, templateCleanupInfo);

            var functionsWithSingleReturn = this.getFunctionsWithSingleReturn();
            redundantFunctions = this.getRedundantFunctions(functionsWithSingleReturn);
            this.pruneFuncReferences(redundantFunctions, templateCleanupInfo);
            iterations++;
        }

        if (hasAstChanges) {
            this.consolidateReturnStatements();
            this.cleanupEmptyConstructor();
            this.emberSpecificCleanup(); //need to separate in a separate file.
        }

        this.finalizeLiterals();

        if (this.print_to_console) {
            if (!this.changed) {
                if (!hasAstChanges) {
                    console.log(colors.yellow(`${this.filename}: Piranha did not make any changes`));
                } else {
                    console.log(
                        colors.green(
                            `${this.filename}: Took ${iterations} ${
                                iterations == 1 ? 'pass' : 'passes'
                            } over the code to reach fixed point.`,
                        ),
                    );
                }
            } else {
                console.log(
                    colors.red(
                        `${this.filename}: Terminated before fixed point in ${iterations} ${
                            iterations == 1 ? 'pass' : 'passes'
                        } over the code.`,
                    ),
                );
            }
        }
        return {
            changed: hasAstChanges,
            templateCleanupInfo,
            hasFlagKeywordInFile,
        };
    }
}

module.exports = {
    // Log refactoring steps to help debug unintended behavior or crashes
    // A person filing an issue can add the output of this logger as a bug report
    logger: winston.createLogger({
        level: 'info',
        format: winston.format.json(),
        transports: [], // Transports are added to this logger in piranha.js when --debug is true
    }),

    RefactorEngine: RefactorEngine,
};
