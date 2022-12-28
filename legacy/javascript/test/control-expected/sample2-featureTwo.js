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

function get() {
    return isToggleDisabled(featureOne);
}

const a = isToggleDisabled(featureOne);

if (!false && isToggleDisabled(featureOne)) {
    // This is the first comment
    console.log('New feature featureTwo is running');
} else {
    // This is another comment
    console.log('Old feature oldFeat2 is running');
}

if (false || isFlagTreated(featureOne)) {
    // This is the second comment
    console.log('New feature featureOne is running');
} else {
    // This is the third comment
    console.log('Old feature oldFeat1 is running');
}

// This is the fourth comment
console.log('New Feature featureTwo is running');

if (true) {
    f();
} else {
    g();
}

// This is the sixth comment