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
 * This module checks the integrity of the source file.
 * In particular, it checks if the file exists and has the correct format.
 *
 * Public API -
 *
 * checkSource -
 * Checks the integrity of a JS source file and reads it to return its contents.
 * Throws errors if the file cannot be accessed or has the incorrect format.
 *
 * @param {String} source_file - path to the *.js source file
 * @returns {String} contents of source file
 */

const fs = require('fs');

module.exports = {
    checkSource: function (source_file) {
        if (!fs.existsSync(source_file)) {
            throw new Error(`File ${source_file} not found`);
        } else if (source_file.split('.').slice(-1)[0] != 'js') {
            throw new Error(`Input ${source_file} is not a javascript file`);
        }

        return source_file;
    },
};
