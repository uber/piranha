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
const refactor = require('../src/refactor');
const fs = require('fs'); // Read and write files
const recast = require('recast'); // Parser
const max_cleanup_steps = 15;
const config_checker = require('../src/config_checker');
const properties = config_checker.parseProperties('config/properties.json');
const babel = require('@babel/core');
const { parseOptions } = require('../config/utils');

var checkForPiranhaLiterals = (ast) => {
    var result = false;

    babel.traverse(ast, {
        exit: function (node) {
            if (node.createdByPiranha !== undefined) {
                result = true;
            }
        },
    });

    return result;
};

describe('piranha', () => {
    describe('#sample.js', () => {
        it('featureFlag treated', () => {
            const code = fs.readFileSync('./test/input/sample.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureFlag';
            const behaviour = true;
            const expected_code = fs.readFileSync('./test/treated-expected/sample-featureFlag.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;
            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('featureFlag control', () => {
            const code = fs.readFileSync('./test/input/sample.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureFlag';
            const behaviour = false;
            const expected_code = fs.readFileSync('./test/control-expected/sample-featureFlag.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });
    });

    describe('#sample1.js', () => {
        it('featureFlag treated', () => {
            const code = fs.readFileSync('./test/input/sample1.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureFlag';
            const behaviour = true;
            const expected_code = fs.readFileSync('./test/treated-expected/sample1-featureFlag.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('featureFlag control', () => {
            const code = fs.readFileSync('./test/input/sample1.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureFlag';
            const behaviour = false;
            const expected_code = fs.readFileSync('./test/control-expected/sample1-featureFlag.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });
    });

    describe('#sample2.js', () => {
        it('featureOne treated', () => {
            const code = fs.readFileSync('./test/input/sample2.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureOne';
            const behaviour = true;
            const expected_code = fs.readFileSync('./test/treated-expected/sample2-featureOne.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('featureOne control', () => {
            const code = fs.readFileSync('./test/input/sample2.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureOne';
            const behaviour = false;
            const expected_code = fs.readFileSync('./test/control-expected/sample2-featureOne.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('featureTwo treated', () => {
            const code = fs.readFileSync('./test/input/sample2.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureTwo';
            const behaviour = true;
            const expected_code = fs.readFileSync('./test/treated-expected/sample2-featureTwo.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });

        it('featureTwo control', () => {
            const code = fs.readFileSync('./test/input/sample2.js', 'utf-8');
            const ast = recast.parse(code, parseOptions);
            const flagname = 'featureTwo';
            const behaviour = false;
            const expected_code = fs.readFileSync('./test/control-expected/sample2-featureTwo.js', 'utf-8');

            const engine = new refactor.RefactorEngine(
                ast,
                properties,
                behaviour,
                flagname,
                max_cleanup_steps,
                false,
                true,
            );
            engine.refactorPipeline();
            const refactored_code = recast.print(ast).code;

            assert(!checkForPiranhaLiterals(ast));
            assert(
                `${expected_code}\n` === refactored_code,
                `\nEXPECTED : ${JSON.stringify(expected_code)}\nREFACTORED : ${JSON.stringify(refactored_code)}`,
            );
        });
    });
});
