# Copyright (c) 2022 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.



from polyglot_piranha import run_piranha_cli
from os.path import join, basename
from os import listdir


def test_piranha_rewrite():
    output_summary = run_piranha_cli('test-resources/java/feature_flag_system_1/treated/input', 'test-resources/java/feature_flag_system_1/treated/configurations', False)
    assert is_as_expected('test-resources/java/feature_flag_system_1/treated', output_summary)

def test_piranha_match_only():
    output_summary = run_piranha_cli('test-resources/java/structural_find/input', 'test-resources/java/structural_find/configurations', False)
    assert len(output_summary[0].matches) == 20

def is_as_expected(path_to_scenario, output_summary):
    expected_output = join(path_to_scenario, 'expected')
    for file_name in listdir(expected_output):
        with open(join(expected_output, file_name), 'r') as f:
            file_content = f.read()
            expected_content = ''.join(file_content.split())
            updated_content = [''.join(o.content.split()) for o in output_summary if basename(o.path) == file_name][0]
            if expected_content != updated_content:
                return False
    return True
