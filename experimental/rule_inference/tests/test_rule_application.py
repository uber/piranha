import pytest
import os
import toml
from rule_inference.rule_application import CodebaseRefactorer


@pytest.fixture
def refactorer():
    language = "java"
    path_to_codebase = "./tests/test_resources/test_codebase"
    path_to_rules = "./tests/test_resources/test_rules.toml"
    include_paths = ["./tests/test_resources/test_codebase/to_include"]
    exclude_paths = ["./tests/test_resources/test_codebase/to_exclude"]
    return CodebaseRefactorer(
        language, path_to_codebase, path_to_rules, include_paths, exclude_paths
    )


def test_create_rule_graph(refactorer):
    # Load the rules from the .toml file
    with open(refactorer.path_to_rules, "r") as file:
        toml_dict = toml.load(file)

    rules = toml_dict.get("rules", [])
    rule_graph = refactorer.create_rule_graph(rules)

    assert len(rule_graph.rules) == len(rules)
    for rule in rule_graph.rules:
        assert rule.name in [rule["name"] for rule in rules]


def test_refactor_codebase(refactorer):
    # Call the method under test
    output_summaries = refactorer.refactor_codebase()
    assert len(output_summaries) == 1
