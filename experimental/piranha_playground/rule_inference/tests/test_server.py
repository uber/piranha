import json
import pathlib
from unittest.mock import patch

import pytest

from piranha_playground.main import app
from piranha_playground.rule_inference.rule_application import (
    _run_piranha_with_timeout_aux,
)
from piranha_playground.rule_inference.utils.rule_utils import *


@pytest.fixture
def client():
    app.config["TESTING"] = True
    with app.test_client() as client:
        yield client


def test_infer_static_rule(client):
    # Test data for the /infer_piranha route
    data = {
        "source_code": "// 1\nclass A {}",
        "target_code": "// 1\nclass B {}",
        "language": "java",
    }
    response = client.post("/infer_rule_graph", json=data)

    assert response.status_code == 200
    graph = RawRuleGraph.from_toml(toml.loads(response.get_json()["rules"]))
    assert len(graph.rules) == 1


@pytest.mark.skip(reason="This test requires OpenAI API key")
def test_improve_rules(client):
    # Test data for the /improve_piranha route
    data = {
        "source_code": "// 1\nclass A {}",
        "target_code": "// 1\nclass B {}",
        "language": "java",
        "requirements": "Can you complete my rule?",
        "rules": '''[[rules]]\nname = "rename_class_A_to_B"''',
        "option": "general",  # Please provide example option data
    }
    response = client.post("/improve_rule_graph", json=data)

    assert response.status_code == 200
    # Check that the response contains rule and gpt_output
    assert "rule" in response.get_json()
    assert "gpt_output" in response.get_json()


def test_test_rule(client):
    # Test data for the /test_rule route

    rule = '''[[rules]]
        name = "rename_class_A_to_B"
        query = """    (class_declaration
                name: (identifier) @class_name
                (#eq? @class_name "A"))
        """
        replace_node = "class_name"
        replace = """B"""
        '''

    data = {"source_code": "class A {}", "language": "java", "rules": rule}
    with patch(
        "piranha_playground.rule_inference.rule_application.run_piranha_with_timeout",
        new=_run_piranha_with_timeout_aux,
    ):
        response = client.post("/test_rule", json=data)

    assert response.status_code == 200
    # Check that the response contains refactored_code
    assert response.get_json()["refactored_code"] == "class B {}"


def test_process_folder(client):
    # Setup rules and edges from toml files
    rules = pathlib.Path(
        "test-resources/java/feature_flag_system_2/control/configurations/rules.toml"
    ).read_text()
    edges = pathlib.Path(
        "test-resources/java/feature_flag_system_2/control/configurations/edges.toml"
    ).read_text()

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

    # Test data for the /refactor_codebase route
    data = {
        "language": "java",
        "folder_path": "test-resources/java/feature_flag_system_2/control/input",
        "rules": toml.dumps(toml_dict),
    }

    with patch(
        "piranha_playground.rule_inference.rule_application.run_piranha_with_timeout",
        new=_run_piranha_with_timeout_aux,
    ):
        response = client.post("/refactor_codebase", json=data)

    # Check status code and 'result' field in response
    assert response.status_code == 200
    assert "result" in response.get_json()
    assert response.get_json()["result"] == True
