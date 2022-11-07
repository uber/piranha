/*
Copyright (c) 2022 Uber Technologies, Inc.
 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0
 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

for (let i = 0; i < 10; ++i) {
    console.log(i);
}

function f1() {
    for (let i = 0; i < 10; ++i) {
        console.log(i);
    }
}

function f2() {
    let i = 0;

    while(i < 10) {
        for (let j = 0; j < 10; ++j) {
            console.log(10*i + j);
        }

        ++i;
    }
}

