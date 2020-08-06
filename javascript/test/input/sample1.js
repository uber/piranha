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

// JS style imports
import 'module1';
import 'module2';
import 'module3';

// CommonJS style imports
const module4 = require('module4');
const module5 = require('module5');

for (var i = 0; i == 0; i++) {
    // Simple flag cleanup in conditional
    if (isFlagTreated(featureFlag)) {
        f1();
    } else {
        f2();
    }
}

// Deep clean of flag
var a = isToggleDisabled(featureFlag)
    ? f1()
    : isToggleDisabled(featureFlag)
    ? f2()
    : isFlagTreated(featureFlag)
    ? f3()
    : f4();

// Assignment cleanup
var b = isToggleDisabled(featureFlag);

// conditional cleanup
let c = isFlagTreated(featureFlag) && b;

// function cleanup
function d() {
    return isFlagTreated(featureFlag);
}

let e = true && d();
