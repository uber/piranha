#!/bin/bash
# Copyright (c) 2022 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

if [ $0 != "./demo/run_piranha_demo.sh" ]; then
    echo "Please run the script from 'piranha/polyglot/piranha'"
    exit 1
fi

if [ $# -eq 0 ]; then
    echo "Please provide a target language or 'ALL' as argument."
    exit 1
fi

demo_directories=()
if [ $1 = "ALL" ]; then
    for language_directory in "demo/*/"; do
        demo_directories+=$language_directory
    done
else
    demo_dir="demo/${1}/"
    if [ ! -d "$demo_dir" ]; then
        echo "No demo for language '${1}'"
        exit 1
    fi
    demo_directories+=$demo_dir
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    # Mac OSX
    cargo build --release --no-default-features
else
    cargo build --release
fi

mv target/release/polyglot_piranha demo/polyglot_piranha

for demo_directory in $demo_directories; do
    ./demo/polyglot_piranha -c $demo_directory -f "$demo_directory/configurations" -j "$demo_directory/output.json"
done
