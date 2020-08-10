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

// Simple flag cleanup in conditional
f2();

// ---------------------------------

// Assignment cleanup
f1();

// ----------------------------------

// function cleanup

if (f1()) {
    f1();
} else {
    f2();
}

// ----------------------------------

// Complex cleanup
var c = f1();

// Literal conversion
console.log(false);