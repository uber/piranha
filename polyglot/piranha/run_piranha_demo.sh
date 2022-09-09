#!/bin/bash
# Copyright (c) 2022 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

if [[ "$OSTYPE" == "darwin"* ]]; then
    # Mac OSX
    cargo build --release --no-default-features
else
    cargo build --release
fi

mv target/release/polyglot_piranha demo/polyglot_piranha
./demo/polyglot_piranha -c demo/java/ -f demo/java/configurations -j demo/java/output.json
./demo/polyglot_piranha -c demo/kt/ -f demo/kt/configurations -j demo/kt/output.json
./demo/polyglot_piranha -c demo/swift/ -f demo/swift/configurations -j demo/swift/output.json
./demo/polyglot_piranha -c demo/strings/ -f demo/strings/configurations -j demo/strings/output.json
