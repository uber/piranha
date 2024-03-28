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
from polyglot_piranha import PiranhaArguments, execute_piranha
from tree_sitter import Language, Node, Parser
from tree_sitter_languages import get_language, get_parser

from piranha_playground.rule_inference.static_inference import Inference
from piranha_playground.rule_inference.utils.rule_utils import RawRuleGraph


def parse_code(language: str, source_code: str, target_code: str) -> Tuple[Node, Node]:
    """Helper function to parse into tree sitter nodes
    note for gpt: add type annotations, reST docstrings, and type hints"""
    parser = get_parser(language)

    source_tree = parser.parse(bytes(source_code, "utf8"))
    target_tree = parser.parse(bytes(target_code, "utf8"))

    source_node = source_tree.root_node
    target_node = target_tree.root_node
    return source_node, target_node


def test_static_infer_total_replacement():
    # Source and target code samples
    language = "java"
    source_code = "if (flag) { doSomething(); } else { doSomethingElse(); }"
    target_code = "{ doSomething(); }"

    # Initialize parser and parse the code
    source_node, target_node = parse_code(language, source_code, target_code)

    # Instantiate the Inference class
    inference = Inference([source_node], [target_node])

    # Run static inference
    rule = inference.static_infer()
    rule.query = rule.query.replace(
        "parenthesized_expression", "condition"
    )  # Hack because of the mismatch between piranha's parser and tree-sitter's parser
    raw_graph = RawRuleGraph([rule], [])

    # Run the rule on the source code with piranha
    args = PiranhaArguments(
        code_snippet=source_code,
        language=language,
        rule_graph=raw_graph.to_graph(),
    )

    piranha_results = execute_piranha(args)
    # Check if the execution returns results, if yes then return the content of the first result
    # Otherwise, return an empty list
    assert piranha_results[0].content == target_code


def test_static_infer_partial_removal():
    language = "java"
    source_code = "public class Main { public static void main(String[] args) { System.out.println(flag); flag = 5; } }"
    target_code = "public class Main { public static void main(String[] args) { System.out.println(flag); } }"

    source_node, target_node = parse_code(language, source_code, target_code)

    inference = Inference([source_node], [target_node])
    rule = inference.static_infer()

    raw_graph = RawRuleGraph([rule], [])
    args = PiranhaArguments(
        code_snippet=source_code, language=language, rule_graph=raw_graph.to_graph()
    )
    piranha_results = execute_piranha(args)

    assert piranha_results[0].content == target_code


def test_static_infer_full_removal():
    language = "java"
    source_code = "public class Main { public static void main(String[] args) { System.out.println(flag); flag = 5; } }"

    source_node, _ = parse_code(language, source_code, "")

    inference = Inference([source_node], [])
    rule = inference.static_infer()

    raw_graph = RawRuleGraph([rule], [])
    args = PiranhaArguments(
        code_snippet=source_code, language=language, rule_graph=raw_graph.to_graph()
    )
    piranha_results = execute_piranha(args)

    assert piranha_results[0].content == ""


def test_rule_simplification():
    language = "java"
    source_code = "public class Main { public static void main(String[] args) { System.out.println(flag); } }"
    target_code = 'public class Main { public static void main(String[] args) { System.out.println("replaced"); } }'

    source_node, target_node = parse_code(language, source_code, target_code)

    inference = Inference([source_node], [target_node])
    rule = inference.static_infer()

    assert (
        rule.query == """((identifier ) @tag1n\n(#eq? @tag1n "flag"))"""
        and rule.replace_node == """tag1n"""
        and rule.replace == """\" replaced \""""
    )
