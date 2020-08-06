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
const config_checker = require('../src/config_checker');
const parseProperties = config_checker.parseProperties;

describe('config_checker', () => {
    describe('#parseProperties', () => {
        it('should throw error on invalid properties filepath.', () => {
            expect(parseProperties.bind(parseProperties, 'random_prop_file_that_does_not_exist.json', true)).to.throw(
                'File random_prop_file_that_does_not_exist.json not found',
            );
        });

        it('should throw error on invalid json syntax.', () => {
            expect(
                parseProperties.bind(parseProperties, 'test/wrong_properties/wrong_properties1.json', true),
            ).to.throw('test/wrong_properties/wrong_properties1.json does not follow JSON syntax');
        });

        it('should throw error on missing keys in some method.', () => {
            expect(
                parseProperties.bind(parseProperties, 'test/wrong_properties/wrong_properties2.json', true),
            ).to.throw(
                `{"methodName":"isToggleEnabled","flagType":"treated"} in test/wrong_properties/wrong_properties2.json doesn't have 'argumentIndex'`,
            );
        });

        it('should throw error on missing key methodProperties.', () => {
            expect(
                parseProperties.bind(parseProperties, 'test/wrong_properties/wrong_properties3.json', true),
            ).to.throw('The methodProperties property is missing in test/wrong_properties/wrong_properties3.json');
        });
    });
});
