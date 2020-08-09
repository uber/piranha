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
const recast = require('recast');

describe('comments', () => {
    describe('#collateCommentsByPosition', () => {
        it('should collate given comments', () => {
            let code = `//a\n{\n\t//b\n}\n//c`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let leadingComments, trailingComments, remainingComments;
            [leadingComments, trailingComments, remainingComments] = engine.collateCommentsByPosition(ast.body[0]);
            assert(
                leadingComments.length == 1 &&
                    leadingComments[0].value == 'a' &&
                    trailingComments.length == 1 &&
                    trailingComments[0].value == 'c' &&
                    remainingComments.length == 1 &&
                    remainingComments[0].value == 'b',
            );
        });

        it('should collate given comments', () => {
            let code = `//a\nvar a = true //b\n//c`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let leadingComments, trailingComments, remainingComments;
            [leadingComments, trailingComments, remainingComments] = engine.collateCommentsByPosition(ast.body[0]);
            assert(
                leadingComments.length == 1 &&
                    leadingComments[0].value == 'a' &&
                    trailingComments.length == 2 &&
                    trailingComments[0].value == 'b' &&
                    trailingComments[1].value == 'c' &&
                    remainingComments.length == 0,
            );
        });
    });

    describe('#attachCommentsAtEnd', () => {
        it('should attach comments at the end of previously present comments', () => {
            let code = `//a\nvar a = true; //b\n//c\nvar b = true; //d`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let node = ast.body[0];
            engine.attachCommentsAtEnd(node, ast.body[1].comments);
            let comm = ast.body[0].comments;
            assert(
                comm.length == 4 &&
                    comm[0].value == 'a' &&
                    comm[1].value == 'b' &&
                    comm[2].value == 'c' &&
                    comm[3].value == 'd',
            );
        });
    });

    describe('#attachCommentsAtBeginning', () => {
        it('should attach comments in front of previously present comments', () => {
            let code = `//a\nvar a = true; //b\n//c\nvar b = true; //d`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let node = ast.body[0];
            engine.attachCommentsAtBeginning(node, ast.body[1].comments);
            let comm = ast.body[0].comments;
            assert(
                comm.length == 4 &&
                    comm[0].value == 'c' &&
                    comm[1].value == 'd' &&
                    comm[2].value == 'a' &&
                    comm[3].value == 'b',
            );
        });
    });

    describe('#flipCommentPosition', () => {
        it('should flip leading comments to trailing and vice versa', () => {
            let code = `//a\nvar a = true; //b`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let node = ast.body[0];
            node.comments.map(engine.flipCommentPosition);
            let comm = ast.body[0].comments;
            assert(comm.length == 2 && comm[0].trailing && comm[1].leading);
        });

        it("shouldn't flip comment not attached to any node", () => {
            let code = `{\n\t//a\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            let node = ast.body[0];
            node.comments.map(engine.flipCommentPosition);
            let comm = ast.body[0].comments;
            assert(comm.length == 1 && !comm[0].trailing && !comm[0].leading);
        });
    });

    describe('#moveAllCommentsToSiblings', () => {
        it('should attach leading comments to previous sibling and trailing to next sibling', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveAllCommentsToSiblings(ast.body[0].body[1], ast.body[0]);
            let comm0 = ast.body[0].body[0].comments;
            let comm2 = ast.body[0].body[2].comments;
            assert(comm0.length == 3 && comm2.length == 3 && comm0[2].value == 'c' && comm2[0].value == 'd');
        });

        it('should attach leading and trailing comments to next sibling if previous sibling is missing', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveAllCommentsToSiblings(ast.body[0].body[0], ast.body[0]);
            let comm1 = ast.body[0].body[1].comments;
            assert(comm1.length == 4 && comm1[0].value == 'a' && comm1[1].value == 'b');
        });

        it('should attach leading and trailing comments to previous sibling if next sibling is missing', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveAllCommentsToSiblings(ast.body[0].body[2], ast.body[0]);
            let comm1 = ast.body[0].body[1].comments;
            assert(comm1.length == 4 && comm1[2].value == 'e' && comm1[3].value == 'f');
        });
    });

    describe('#moveLeadingCommentsToSibling', () => {
        it('should attach leading comments to previous sibling', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveLeadingCommentsToSibling(ast.body[0].body[1], ast.body[0]);
            let comm0 = ast.body[0].body[0].comments;
            let comm2 = ast.body[0].body[2].comments;
            assert(comm0.length == 2 && comm2.length == 3 && comm2[0].value == 'c');
        });

        it('should attach leading comments to next sibling if previous sibling is missing', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveLeadingCommentsToSibling(ast.body[0].body[2], ast.body[0]);
            let comm1 = ast.body[0].body[1].comments;
            assert(comm1.length == 3 && comm1[2].value == 'e');
        });
    });

    describe('#moveCommentsToConsequent', () => {
        it('should attach comments of IfStatement to its consequent', () => {
            let code = `//a\nif(true){var a = true;}else{var b = true;}//b`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveCommentsToConsequent(ast.body[0]);
            let comm = ast.body[0].consequent.comments;
            assert(comm.length == 2 && comm[0].value == 'a' && comm[1].value == 'b');
        });
    });

    describe('#moveCommentsToAlternate', () => {
        it('should attach comments of IfStatement to its alternate', () => {
            let code = `//a\nif(false){var a = true;}else{var b = true;}//b`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveCommentsToAlternate(ast.body[0]);
            let comm = ast.body[0].alternate.comments;
            assert(comm.length == 2 && comm[0].value == 'a' && comm[1].value == 'b');
        });
    });

    describe('#moveCommentsToExtremeChildren', () => {
        it('should attach comments of BlockStatement to extreme children', () => {
            let code = `//a\n{\n\tvar a = true;\n}//b`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveCommentsToExtremeChildren(ast.body[0], ast, true);
            let comm = ast.body[0].body[0].comments;
            assert(comm.length == 2 && comm[0].value == 'a' && comm[1].value == 'b');
        });

        it('should attach comments to parents if body is empty', () => {
            let code = `//a\n{\n\t//c\n}//b`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.moveCommentsToExtremeChildren(ast.body[0], ast, true);
            let comm = ast.body[0].comments;
            assert(comm.length == 3 && comm[0].value == 'a' && comm[1].value == 'c' && comm[2].value == 'b');
        });
    });

    describe('#preserveCommentsBasedOnOption', () => {
        it('should attach comments based on option', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.preserveCommentsBasedOnOption(ast.body[0].body[1], ast.body[0], true);
            let comm0 = ast.body[0].body[0].comments;
            let comm2 = ast.body[0].body[2].comments;
            assert(comm0.length == 3 && comm2.length == 3 && comm0[2].value == 'c' && comm2[0].value == 'd');
        });

        it('should attach comments based on option', () => {
            let code = `{\n\t//a\n\tvar a = true; //b\n\t//c\n\tvar b = true; //d\n\t//e\n\tvar c = true; //f\n}`;

            let ast = recast.parse(code).program;
            let engine = new refactor.RefactorEngine(ast);
            engine.preserveCommentsBasedOnOption(ast.body[0].body[1], ast.body[0], false);
            let comm0 = ast.body[0].body[0].comments;
            let comm2 = ast.body[0].body[2].comments;
            assert(comm0.length == 2 && comm2.length == 3 && comm2[0].value == 'c');
        });
    });
});
