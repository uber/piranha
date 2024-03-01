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

const assert = require('assert');
const properties = require('../config/properties.json');
const refactor = require('../src/refactor');
const recast = require('recast');
const { parseOptions } = require('../config/utils');

// Constants used in all experiments
const treated = true;
const flagname = 'testFlag';

// Boolean literals followed by [o] are present in the original code and not introduced by Piranha

describe('refactor', () => {
    describe('#flagAPIToLiteral', () => {
        it('should introduce the createdByPiranha property on the literals', () => {
            const code = 'isFlagTreated(testFlag)';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();

            assert(ast.program.body[0].expression.createdByPiranha !== undefined);
        });

        const boolean_literal_replacement_specs = [
            {
                name: 'Identifier',
                code: `if (isFlagTreated(testFlag)) {} else {}`,
                expected_code: 'if (true) {} else {}',
            },
            {
                name: 'StringLiteral',
                code: `if (isFlagTreated('testFlag')) {} else {}`,
                expected_code: 'if (true) {} else {}',
            },
        ];

        boolean_literal_replacement_specs.forEach((spec) => {
            it(`should replace ${spec.name} calls to flag check APIs with boolean literals`, () => {
                const ast = recast.parse(spec.code, parseOptions);

                const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
                engine.flagAPIToLiteral();
                engine.finalizeLiterals();

                const refactored_code = recast.print(ast).code;

                assert(engine.changed);
                assert(
                    spec.expected_code === refactored_code,
                    `\nEXPECTED : ${JSON.stringify(spec.expected_code)}\nREFACTORED : ${JSON.stringify(
                        refactored_code,
                    )}`,
                );
            });
        });
    });

    describe('#evalBoolExpressions', () => {
        it('should replace true && f() with just f()', () => {
            const code = `if (isFlagTreated(testFlag) && f()) {} else {}`;
            const expected_code = `if (f()) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace false && f() with just false', () => {
            const code = `if (isToggleDisabled(testFlag) && f()) {} else {}`;
            const expected_code = `if (false) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace true || f() with just true', () => {
            const code = `if (isFlagTreated(testFlag) || f()) {} else {}`;
            const expected_code = `if (true) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace false || f() with just f()', () => {
            const code = `if (isToggleDisabled(testFlag) || f()) {} else {}`;
            const expected_code = `if (f()) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace !false with just true', () => {
            const code = `if (!isToggleDisabled(testFlag)) {} else {}`;
            const expected_code = `if (true) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace !true with just false', () => {
            const code = `if (!isFlagTreated(testFlag)) {} else {}`;
            const expected_code = `if (false) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace !(!(f() || true)) with just true', () => {
            const code = `if (!(!(f() || isFlagTreated(testFlag)))) {} else {}`;
            const expected_code = `if (true) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should leave f() && g() unchanged', () => {
            const code = `if (f() && g()) {} else {}`;
            const expected_code = `if (f() && g()) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(!engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should leave f() && true[o] unchanged', () => {
            const code = `if (f() && true) {} else {}`;
            const expected_code = `if (f() && true) {} else {}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.evalBoolExpressions();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(!engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });
    });

    describe('#reduceIfStatements', () => {
        it('should leave if(true[o]) {f(); h();} else {g()} unchanged', () => {
            const code = `if (true) {f(); h();} else {g()}`;
            const expected_code = `if (true) {f(); h();} else {g()}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(!engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(true) {f(); h();} else {g()} with f(); h();', () => {
            const code = `if (isFlagTreated(testFlag)) {f(); h();} else {g()}`;
            const expected_code = `f();h();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(true) {f(); h();} with f(); h();', () => {
            const code = `if (isFlagTreated(testFlag)) {f(); h();}`;
            const expected_code = `f();h();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(true) f(); else g(); with f();', () => {
            const code = `if(isFlagTreated(testFlag)) f(); else g();`;
            const expected_code = `f();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(false) f(); else if (true) {g(); g_();} else h(); with g(); g_();', () => {
            const code = `if(isToggleDisabled(testFlag)) f(); else if (isFlagTreated(testFlag)) {g(); g_();} else h();`;
            const expected_code = `g();g_();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('if(false) f(); else if (false) g(); else if (false) h(); else if (false) i(); else j(); with j()', () => {
            const code = `if(isToggleDisabled(testFlag)) f(); else if (isToggleDisabled(testFlag)) g(); else if (isToggleDisabled(testFlag)) h(); else if (isToggleDisabled(testFlag)) i(); else j();`;
            const expected_code = `j();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('if(false) {f();} else if (false) {g();} else if (false) h(); else if (false) {i();} else j(); with j()', () => {
            const code = `if(isToggleDisabled(testFlag)) {f();} else if (isToggleDisabled(testFlag)) {g();} else if (isToggleDisabled(testFlag)) h(); else if (isToggleDisabled(testFlag)) {i();} else j();`;
            const expected_code = `j();`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(false) {f()} else {g()} with g()', () => {
            const code = `if (isToggleDisabled(testFlag)) {f()} else {g()}`;
            const expected_code = `g()`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should replace if(f1()) {f2()} else if (true) {g()} with g();', () => {
            const code = `if(f1()) \n{f2();} else if (isFlagTreated(testFlag)) {g();}`;
            const expected_code = `if (f1())\n  {f2();} else {\n  g();\n}`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should leave if(h()) {f()} else {g()} unchanged', () => {
            const code = `if (h()) {f()} else {g()}`;
            const expected_code = code;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(!engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('should simplify ternary expression', () => {
            const code = `var b = isFlagTreated(testFlag) ? "hi" : "hello";`;
            const expected_code = `var b = "hi";`;
            const ast = recast.parse(code, parseOptions);

            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;

            assert(engine.changed);
            assert(
                expected_code === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });
    });

    describe('#getRedundantVarnames', () => {
        it('should return variables which are set to a stale flag', () => {
            const code = `a = isFlagTreated(testFlag); var b = isFlagTreated(testFlag);`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();

            const assignments = engine.getRedundantVarnames();
            const expected_assignments = { a: true, b: true };

            assert(JSON.stringify(expected_assignments) === JSON.stringify(assignments));
        });

        it('should return const variables which are set to a stale flag', () => {
            const code = `const a = isFlagTreated(testFlag); var b; b = isFlagTreated(testFlag);`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();

            const assignments = engine.getRedundantVarnames();
            const expected_assignments = { a: true, b: true };

            assert(JSON.stringify(expected_assignments) === JSON.stringify(assignments));
        });

        it('should not include variables initialized to null', () => {
            const code = `var b = null`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            const assignments = engine.getRedundantVarnames();
            const expected_assignments = {};

            assert(JSON.stringify(expected_assignments) === JSON.stringify(assignments));
        });

        it('should not include uninitialized variables', () => {
            const code = `var b`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();

            const assignments = engine.getRedundantVarnames();
            const expected_assignments = {};

            assert(JSON.stringify(expected_assignments) === JSON.stringify(assignments));
        });

        it('should return list of variable names which are set to a stale flag', () => {
            const code = `var b = true`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();

            const assignments = engine.getRedundantVarnames();
            const expected_assignments = {};

            assert(JSON.stringify(expected_assignments) === JSON.stringify(assignments));
        });
    });

    describe('#replaceOrDeleteReferences', () => {
        it('should leave var a = true[o] unchanged', () => {
            const code = `var a = true; var b = a;`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneVarReferences(engine.getRedundantVarnames());
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var a = true; var b = a;`;

            assert(!engine.changed);
            assert(refactored_code === expected_code);
        });

        it('should replace or delete references of vars set to stale flag', () => {
            const code = `var a = isFlagTreated(testFlag); var b = a;`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneVarReferences(engine.getRedundantVarnames());
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var b = true;`;

            assert(refactored_code === expected_code);
        });

        it('should replace or delete references of vars set to stale flag', () => {
            const code = `var a = true; var b = isFlagTreated(testFlag);`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneVarReferences(engine.getRedundantVarnames());
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var a = true;`;

            assert(refactored_code === expected_code);
        });

        it('should replace or delete references of vars set to stale flag', () => {
            const code = `let a; a = isFlagTreated(testFlag); const b = a;`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneVarReferences(engine.getRedundantVarnames());
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `const b = true;`;

            assert(refactored_code === expected_code);
        });
    });

    describe('#getRedundantFunctions', () => {
        it('function declaration with function keyword - should return a and b', () => {
            const code = `function a(){ return isFlagTreated(testFlag); } function b(){ return false; }`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            const redundant_functions = engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn());
            const expected_functions = { a: true };

            assert(
                JSON.stringify(expected_functions) === JSON.stringify(redundant_functions),
                `\nEXPECTED : ${JSON.stringify(expected_functions)}\nREFACTORED : ${JSON.stringify(
                    redundant_functions,
                )}`,
            );
        });

        it('should get a list of redundant functions which return literal.', () => {
            const code = `const a = function(){ return isFlagTreated(testFlag); } \n var b = () => true`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            const redundant_functions = engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn());
            const expected_functions = { a: true };

            assert(
                JSON.stringify(expected_functions) === JSON.stringify(redundant_functions),
                `\nEXPECTED : ${JSON.stringify(expected_functions)}\nREFACTORED : ${JSON.stringify(
                    redundant_functions,
                )}`,
            );
        });

        it('should get a list of redundant functions which return literal.', () => {
            const code = `const a = (x,y) => { const z = x + y; return isFlagTreated(testFlag); }`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            const redundant_functions = engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn());

            const expected_functions = { a: true };

            assert(JSON.stringify(expected_functions) === JSON.stringify(redundant_functions));
        });

        it('should get a list of redundant functions which return literal.', () => {
            const code = `function a(){ if(false) return true; if (isToggleDisabled(testFlag)) var b = isFlagTreated(testFlag); }`;
            const flagname = 'testFlag';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.reduceIfStatements();
            const redundant_functions = engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn());

            const expected_functions = {};

            assert(
                JSON.stringify(expected_functions) === JSON.stringify(redundant_functions),
                `\nEXPECTED : ${JSON.stringify(expected_functions)}\nREFACTORED : ${JSON.stringify(
                    redundant_functions,
                )}`,
            );
        });
    });

    describe('#pruneFuncReferences', () => {
        it('should leave functions which return boolean literals to begin with unchanged', () => {
            const code = `function a(){ return true; } \n var b = a();`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneFuncReferences(engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn()));
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `function a(){ return true; } \n var b = a();`;

            assert(!engine.changed);
            assert(refactored_code === expected_code);
        });

        it('should replace or delete redundant function declarations and calls.', () => {
            const code = `function a(){ return isFlagTreated(testFlag); } \n var b = a();`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneFuncReferences(engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn()));
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var b = true;`;

            assert(refactored_code === expected_code);
        });

        it('should replace or delete redundant function declarations and calls.', () => {
            const code = `var a = () => isFlagTreated(testFlag) \n var b = a();`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneFuncReferences(engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn()));
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var b = true;`;

            assert(refactored_code === expected_code);
        });

        it('should replace or delete redundant function declarations and calls.', () => {
            const code = `const a = function (){ return isFlagTreated(testFlag); }\nvar b = a();`;

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.pruneFuncReferences(engine.getRedundantFunctions(engine.getFunctionsWithSingleReturn()));
            engine.finalizeLiterals();

            const refactored_code = recast.print(ast).code;
            const expected_code = `var b = true;`;

            assert(refactored_code === expected_code);
        });
    });

    describe('#finalizeLiterals', () => {
        it('should remove createdByPiranha property from a literal', () => {
            const code = 'isFlagTreated(testFlag)';

            let ast = recast.parse(code, parseOptions);
            const engine = new refactor.RefactorEngine(ast, properties, treated, flagname);
            engine.flagAPIToLiteral();
            engine.finalizeLiterals();

            assert(ast.program.body[0].expression.createdByPiranha === undefined);
        });
    });
});
