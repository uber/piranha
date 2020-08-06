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

const chai = require('chai');
const expect = chai.expect;
const source_checker = require('../src/source_checker');
const checkSource = source_checker.checkSource;

describe('source_checker', () => {
    describe('#checkSource', () => {
        it('should throw error on invalid source filepath.', () => {
            expect(checkSource.bind(checkSource, 'random_source_file_that_does_not_exist.js', true)).to.throw(
                'File random_source_file_that_does_not_exist.js not found',
            );
        });

        it('should throw error if source file is not JS.', () => {
            // fs module uses process.cwd() to figure relative path so `npm test` run in root wont't need ../
            expect(checkSource.bind(checkSource, 'config/properties.json', true)).to.throw(
                'Input config/properties.json is not a javascript file',
            );
        });
    });
});
