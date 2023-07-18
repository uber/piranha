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

from typing import Tuple

import pytest
import toml
from polyglot_piranha import PiranhaArguments, execute_piranha
from tree_sitter import Language, Node, Parser
from tree_sitter_languages import get_language, get_parser

from piranha_playground.rule_inference.piranha_agent import PiranhaAgent
from piranha_playground.rule_inference.static_inference import Inference
from piranha_playground.rule_inference.utils.rule_utils import RawRuleGraph


def test_go_inference():
    # Source and target code samples
    language = "go"
    source_code = """
    func main() {
      // 1
      age := :[x]
      // end
    }
    """
    target_code = """
    func main() {
      // 1
      
      // end
    }
    """

    # Initialize parser and parse the code
    agent = PiranhaAgent(source_code, target_code, language)
    res = agent.infer_rules_statically()
    graph = RawRuleGraph.from_toml(toml.loads(res))
    expected = [
        "((short_var_declaration",
        " left: (expression_list ",
        "  (identifier ) @tag3n) @tag2n",
        " right: (_) @tag4n) @tag1n",
        '(#eq? @tag3n "age"))',
    ]
    assert len(graph.rules) == 1 and graph.rules[0].query == "\n".join(expected)


def test_java_inference():
    # Source and target code samples
    language = "java"
    source_code = """ // 1
    :[x: identifier, method_invocation].doSomething(:[y]); 
    """
    target_code = """ // 1
    doSomething(:[x: identifier, method_invocation], :[y]); 
    """

    # Initialize parser and parse the code
    agent = PiranhaAgent(source_code, target_code, language)
    res = agent.infer_rules_statically()
    graph = RawRuleGraph.from_toml(toml.loads(res))
    print(graph.rules[0].query)
    expected = [
        "((method_invocation",
        " object: [(identifier) (method_invocation)] @tag2n",
        " name: (identifier ) @tag3n",
        " arguments: (argument_list ",
        "  (_) @tag5n) @tag4n) @tag1n",
        '(#eq? @tag3n "doSomething"))',
    ]
    assert len(graph.rules) == 1 and graph.rules[0].query == "\n".join(expected)
