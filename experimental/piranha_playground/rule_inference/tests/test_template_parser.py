import re
from collections import defaultdict
from polyglot_piranha import Rule, RuleGraph, PiranhaArguments, execute_piranha
from tree_sitter import Tree
from tree_sitter_languages import get_language, get_parser
import attr
from typing import Tuple, Dict, Optional, List
import pytest
from piranha_playground.rule_inference.template_parser import TemplateParser, WILDCARD



def test_template_parser_init():
    parser = TemplateParser("java")
    assert parser.language == "java"
    assert parser.tree_sitter_language is not None
    assert parser.parser is not None


def test_get_tree_from_code():
    code = 'public class HelloWorld { public static void main(String[] args) { System.out.println("Hello, World!"); } }'
    parser = TemplateParser("java")
    tree = parser.get_tree_from_code(code)
    assert isinstance(tree, Tree)


def test_remove_comments_from_code():
    code = '// This is a line comment\n public class HelloWorld { public static void main(String[] args) { System.out.println("Hello, World!"); } }'
    expected_code = 'public class HelloWorld { public static void main(String[] args) { System.out.println("Hello, World!"); } }'
    parser = TemplateParser("java")
    result_code = parser.remove_comments_from_code(code)
    assert result_code.strip() == expected_code.strip()


def test_replace_template_holes():
    code = "public class :[ClassName] { public static void :[MethodName](String[] args) { System.out.println(:[Message]); } }"
    expected_code = "public class ClassName { public static void MethodName(String[] args) { System.out.println(Message); } }"
    parser = TemplateParser("java")
    result_code = parser.replace_template_holes(code)
    assert result_code == expected_code
    assert parser.template_holes == {
        "ClassName": [WILDCARD],
        "MethodName": [WILDCARD],
        "Message": [WILDCARD],
    }


def test_replace_template_holes_with_specific_nodes():
    code = "public class :[ClassName:identifier] { public static void :[  MethodName :  call_expr, identifier](String[] args) { System.out.println(:[Message]); } }"
    expected_code = "public class ClassName { public static void MethodName(String[] args) { System.out.println(Message); } }"
    parser = TemplateParser("java")
    result_code = parser.replace_template_holes(code)
    assert result_code == expected_code
    print(parser.template_holes)
    assert parser.template_holes == {
        "ClassName": ["identifier"],
        "MethodName": ["call_expr", "identifier"],
        "Message": [WILDCARD],
    }


def test_parse_content():
    parser = TemplateParser("java")
    assert parser.parse_content("ClassName: HelloWorld, Main") == (
        "ClassName",
        ["HelloWorld", "Main"],
    )
    assert parser.parse_content("Message") == ("Message", None)
