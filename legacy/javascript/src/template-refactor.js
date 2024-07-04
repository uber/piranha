const { transform, traverse } = require('ember-template-recast');
const { builders: b } = require('@glimmer/syntax');
const colors = require('colors');

class TemplateRefactorEngine {
    constructor({ ast, flagname, filename, cleanupInfo, print_to_console, properties }) {
        this.ast = ast;
        this.properties = properties;
        this.flagname = flagname;
        this.filename = filename;
        this.cleanupInfo = cleanupInfo;
        this.max_cleanup_steps = 15;
        this.print_to_console = print_to_console;

        this.templateHelpersList = this.properties.templateHelpers.map(({ helperName }) => helperName);
    }

    hasFlagKeywordInFile() {
        let flagname = this.flagname;
        const engine = this;
        let hasFlagKeywordInFile = false;
        traverse(this.ast, {
            All: {
                enter(node) {
                    if (
                        (node.type === 'StringLiteral' && node.value === flagname) ||
                        (node.type === 'PathExpression' && engine.isCleanupProperty(node.original))
                    ) {
                        hasFlagKeywordInFile = true;
                    }
                },
            },
        });
        return hasFlagKeywordInFile;
    }

    trueLiteral() {
        const vanillaLiteral = {
            type: 'BooleanLiteral',
            value: true,
            raw: true,
        };

        vanillaLiteral.createdByPiranha = true;

        return vanillaLiteral;
    }

    falseLiteral() {
        const vanillaLiteral = {
            type: 'BooleanLiteral',
            value: false,
            original: false,
        };

        vanillaLiteral.createdByPiranha = true;

        return vanillaLiteral;
    }

    isPiranhaLiteral(node) {
        return node.type === 'BooleanLiteral' && node.createdByPiranha !== undefined;
    }

    isCleanupProperty(prop) {
        const properties = this.cleanupInfo.properties;
        const thisProperties = this.cleanupInfo.properties.map((prop) => `this.${prop}`);
        return properties.includes(prop) || thisProperties.includes(prop);
    }

    reduceSubExpression(node, operator) {
        let hasFalsePiranhaLiteral = node.params.find(
            (expression) => this.isPiranhaLiteral(expression) && expression.value === false,
        );

        if (operator === 'and') {
            if (hasFalsePiranhaLiteral) {
                return this.falseLiteral();
            } else {
                node.params = node.params.filter((expression) => !this.isPiranhaLiteral(expression));
                if (node.params.length === 1) {
                    return node.params[0];
                }
            }
        } else if (operator === 'or') {
            if (hasFalsePiranhaLiteral) {
                node.params = node.params.filter((expression) => !this.isPiranhaLiteral(expression));
                if (node.params.length === 1) {
                    return node.params[0];
                }
            } else {
                return this.trueLiteral();
            }
        }
    }

    flagAPIToLiteral() {
        const engine = this;
        const flagname = this.flagname;
        const templateHelpersList = this.templateHelpersList;

        transform({
            template: this.ast,
            plugin() {
                return {
                    SubExpression(node) {
                        if (templateHelpersList.includes(node.path.original) && node.params[0].value === flagname) {
                            //{{#if (and (has-temp-feature 'domain_filter') this.a1)}} -> {{#if (and true this.a1)}}
                            engine.changed = true;
                            return engine.trueLiteral();
                        } else if (
                            engine.isCleanupProperty(node.path.original) &&
                            !node.params.length &&
                            !node.hash.pairs.length
                        ) {
                            //{{#if (and (this.hasTempA1Feature) this.a1)}} -> {{#if (and true this.a1)}}
                            engine.changed = true;
                            return engine.trueLiteral();
                        }
                    },
                    PathExpression(node) {
                        // {{#if (and this.hasTempA1Feature this.a1)}} -> {{#if (and true this.a1)}}
                        if (engine.isCleanupProperty(node.original)) {
                            engine.changed = true;
                            return engine.trueLiteral();
                        }
                    },
                };
            },
        });
    }

    evalBoolExpressions() {
        const engine = this;

        traverse(this.ast, {
            All: {
                exit(node) {
                    if (node.type === 'SubExpression') {
                        const operator = node.path.original;

                        if (operator === 'not' && engine.isPiranhaLiteral(node.params[0])) {
                            if (node.params[0].value === true) {
                                engine.changed = true;
                                return engine.falseLiteral();
                            } else if (node.params[0].value === false) {
                                engine.changed = true;
                                return engine.trueLiteral();
                            }
                        } else if (operator === 'and' || operator === 'or') {
                            let hasPiranhaLiteral = node.params.find((expression) =>
                                engine.isPiranhaLiteral(expression),
                            );
                            if (hasPiranhaLiteral) {
                                engine.changed = true;
                                return engine.reduceSubExpression(node, operator);
                            }
                        }
                    }
                },
            },
        });
    }

    reduceIfStatements() {
        const engine = this;

        traverse(this.ast, {
            All: {
                enter(node, path) {
                    if (node.type === 'BlockStatement') {
                        if (
                            node.path.original === 'if' &&
                            node.params.length === 1 &&
                            engine.isPiranhaLiteral(node.params[0])
                        ) {
                            if (node.params[0].value === true) {
                                engine.changed = true;
                                return node.program;
                            } else if (node.params[0].value === false) {
                                if (node.inverse === null) {
                                    engine.changed = true;
                                    return null;
                                } else {
                                    engine.changed = true;

                                    // `if-else`
                                    if (node.inverse.chained === false) {
                                        return node.inverse.body;
                                    }

                                    // `if-elseIf-else`
                                    if (
                                        node.inverse.body[0].inverse &&
                                        node.inverse.body[0].inverse.chained === false
                                    ) {
                                        return b.block(
                                            'if',
                                            node.inverse.body[0].params,
                                            null,
                                            node.inverse.body[0].program,
                                            node.inverse.body[0].inverse,
                                        );
                                    }

                                    // `if-elseIf-elseif...else`
                                    node.params[0] = node.inverse.body[0].params[0];
                                    node.program.body[0].chars = node.inverse.body[0].program.body[0].chars;
                                    node.inverse = node.inverse.body[0].inverse;
                                }
                            }
                        }
                    } else if (node.type === 'MustacheStatement' || node.type === 'SubExpression') {
                        if (
                            node.path.original === 'if' &&
                            node.params.length > 1 &&
                            engine.isPiranhaLiteral(node.params[0])
                        ) {
                            if (node.params[0].value === true) {
                                engine.changed = true;
                                if (
                                    path.parent.node.type === 'ConcatStatement' &&
                                    node.params[1].type === 'StringLiteral'
                                ) {
                                    return {
                                        type: 'TextNode',
                                        chars: node.params[1].value,
                                    };
                                } else {
                                    return node.params[1];
                                }
                            } else if (node.params[0].value === false) {
                                engine.changed = true;
                                if (!node.params[2]) return null;
                                if (
                                    path.parent.node.type === 'ConcatStatement' &&
                                    node.params[2].type === 'StringLiteral'
                                ) {
                                    return {
                                        type: 'TextNode',
                                        chars: node.params[2].value,
                                    };
                                } else {
                                    return node.params[2];
                                }
                            }
                        }
                    }
                },
            },
        });
    }

    reduceUnlessStatements() {
        const engine = this;

        traverse(this.ast, {
            All: {
                enter(node, path) {
                    if (node.type === 'BlockStatement') {
                        if (
                          node.path.original === 'unless' &&
                          node.params.length === 1 &&
                          engine.isPiranhaLiteral(node.params[0])
                        ) {
                            if (node.params[0].value === true) {
                                engine.changed = true;
                                return null;
                            } else if (node.params[0].value === false) {
                                engine.changed = true;
                                return node.program;
                            }
                        }
                    } else if (node.type === 'MustacheStatement' || node.type === 'SubExpression') {
                        if (
                          node.path.original === 'unless' &&
                          node.params.length > 1 &&
                          engine.isPiranhaLiteral(node.params[0])
                        ) {
                            if (node.params[0].value === false) {
                                engine.changed = true;
                                if(path.parent.node.type === 'ConcatStatement' && node.params[1].type === 'StringLiteral') {
                                    return {
                                        type: 'TextNode',
                                        chars: node.params[1].value,
                                    }
                                } else {
                                    return b.mustache(
                                      node.params[1],
                                      [],
                                      b.hash(),
                                      false,
                                      undefined
                                    );
                                }
                            } else if (node.params[0].value === true) {
                                engine.changed = true;
                                if(path.parent.node.type === 'ConcatStatement') {
                                    return null;
                                } else {
                                    return b.mustache(
                                      b.path('null'),
                                      [],
                                      b.hash(),
                                      false,
                                      undefined
                                    );
                                }
                            }
                        }
                    }
                },
            },
        });
    }

    refactorPipeline() {
        const hasFlagKeywordInFile = this.hasFlagKeywordInFile();
        let hasAstChanges = false;

        this.flagAPIToLiteral();
        if (this.changed) hasAstChanges = true;

        let iterations = 0;
        this.changed = true;

        while (this.changed && iterations < 15) {
            this.changed = false;

            this.evalBoolExpressions();
            this.reduceIfStatements();
            this.reduceUnlessStatements();

            iterations++;
        }
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
            hasFlagKeywordInFile,
        };
    }
}

module.exports = {
    TemplateRefactorEngine,
};
