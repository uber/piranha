# Copyright (c) 2023 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.
import pathlib
from unittest.mock import Mock, patch

import pytest
import toml
from polyglot_piranha import PiranhaOutputSummary

from piranha_playground.rule_inference.rule_application import (
    CodebaseRefactorer, CodebaseRefactorerException,
    _run_piranha_with_timeout_aux)


# Test for successful execution of Piranha with a timeout
def test_run_piranha_with_timeout_success():
    rules = pathlib.Path(
        "test-resources/java/feature_flag_system_2/control/configurations/rules.toml"
    ).read_text()
    edges = pathlib.Path(
        "test-resources/java/feature_flag_system_2/control/configurations/edges.toml"
    ).read_text()

    language = "java"
    # append substitutions to the toml_dict
    toml_dict = {
        **toml.loads(rules),
        **toml.loads(edges),
        "substitutions": [
            {
                "stale_flag_name": "STALE_FLAG",
                "treated": "true",
                "treated_complement": "false",
                "namespace": "some_long_name",
            }
        ],
    }

    refactorer = CodebaseRefactorer(
        language,
        "test-resources/java/feature_flag_system_2/control/input",
        toml.dumps(toml_dict),
    )
    output_summaries = refactorer.refactor_codebase()
    assert len(output_summaries) == 4
    summary: PiranhaOutputSummary
    for summary in output_summaries:
        for rewrite in summary.rewrites:
            assert rewrite.matched_rule
            assert rewrite.p_match.matched_string and rewrite.p_match.matches


def test_snippet_application():
    language = "java"
    graph = '''
    [[rules]]
    name = "rename_variable"
    query = """(
        (identifier) @var_name 
        (#eq? @var_name "A")
    )"""
    replace_node = "var_name"
    replace = "B"
    '''
    source_code = "class A {}"

    with patch('piranha_playground.rule_inference.rule_application.run_piranha_with_timeout', new=_run_piranha_with_timeout_aux):
        x = CodebaseRefactorer.refactor_snippet(source_code, language, graph)
        assert x == "class B {}"
