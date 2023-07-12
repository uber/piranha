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
from tree_sitter import Node, Tree
from tree_sitter_languages import get_parser

from piranha_playground.rule_inference.graph_parser import TemplateParser


def parse_code(language: str, source_code: str, target_code: str) -> Tuple[Tree, Tree]:
    """Helper function to parse into tree sitter nodes
    note for gpt: add type annotations, reST docstrings, and type hints"""
    parser = get_parser(language)

    source_tree = parser.parse(bytes(source_code, "utf8"))
    target_tree = parser.parse(bytes(target_code, "utf8"))

    return source_tree, target_tree


def test_simple_template():
    language = "java"

    source_code = """
    class SomeClass {
      // 1
      void someMethod(String arg) {

      }
      // end
    }
    """

    target_code = """
    class SomeClass {
      // 1
      public String someMethod() {

      }
      // end
    }
    """
    # Parse the source and target code
    source_tree, target_tree = parse_code(language, source_code, target_code)

    # Initialize GraphParser
    graph_parser = TemplateParser(source_tree, target_tree)

    # Process trees
    result = graph_parser.parse_templates()
    assert (
        result["1"][0][0].text == b"void someMethod(String arg) {\n\n      }"
        and result["1"][1][0].text == b"public String someMethod() {\n\n      }"
    )


def test_edge_parsing():
    language = "java"
    source_code = """
    class SomeClass {
      // 1 -> 2
      // 2 ->    3
      //2 -> 1
    }
    """
    target_code = """
    class SomeClass {
    }
    """
    # Parse the source and target code
    source_tree, target_tree = parse_code(language, source_code, target_code)

    # Initialize GraphParser
    graph_parser = TemplateParser(source_tree, target_tree)

    # Process trees
    graph_parser.parse_templates()
    assert graph_parser.edges == {"1": {"2"}, "2": {"3", "1"}}


def test_complex_template_non_nested():
    language = "java"

    method_1_src = "void someMethod(String arg) {\n      }"
    method_1_tgt = (
        'public String someMethod() {\n        return "Hello, world!";\n      }'
    )

    method_2_src = "void otherMethod(String arg) {\n        someMethod(arg);\n      }"
    method_2_tgt = (
        "public String otherMethod() {\n        return someMethod();\n      }"
    )

    method_3_src = 'void thirdMethod() {\n        otherMethod("hello");\n      }'
    method_3_tgt = "public void thirdMethod() {\n        String message = otherMethod();\n        System.out.println(message);\n      }"

    source_code = f"""
    class SomeClass {{
      // 1 -> 2
      // 2 -> 3
      
      // 1
      {method_1_src}
      // end

      // 2
      {method_2_src}
      // end

      // 3
      {method_3_src}
      // end
    }}
    """

    target_code = f"""
    class SomeClass {{
      // 1
      {method_1_tgt}
      // end

      // 2
      {method_2_tgt}
      // end

      // 3
      {method_3_tgt}
      // end
    }}
    """

    # Parse the source and target code
    source_tree, target_tree = parse_code(language, source_code, target_code)

    # Initialize GraphParser
    graph_parser = TemplateParser(source_tree, target_tree)

    # Process trees
    result = graph_parser.parse_templates()

    # Assert the expected texts in the processed trees
    assert (
        result["1"][0][0].text == method_1_src.encode()
        and result["1"][1][0].text == method_1_tgt.encode()
        and result["2"][0][0].text == method_2_src.encode()
        and result["2"][1][0].text == method_2_tgt.encode()
        and result["3"][0][0].text == method_3_src.encode()
        and result["3"][1][0].text == method_3_tgt.encode()
        and graph_parser.edges == {"1": {"2"}, "2": {"3"}}
    )
