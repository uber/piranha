import toml
import pathlib
from experimental.rule_inference.rule_application import CodebaseRefactorer
from polyglot_piranha import PiranhaOutputSummary


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
    result, output_summaries = refactorer.refactor_codebase(False)
    assert len(output_summaries) == 4
    summary: PiranhaOutputSummary
    for summary in output_summaries:
        for rewrite in summary.rewrites:
            assert rewrite.matched_rule
            assert rewrite.p_match.matched_string and rewrite.p_match.matches


# Test for timeout scenario while executing Piranha
def test_run_piranha_with_timeout_exception():
    language = "java"
    graph = """
    [[rules]]
    name = "rename_variable"
    query = "(identifier) @var_name"
    replace_node = "var_name"
    replace = "other_name"
    """
    source_code = "class A {}"

    success, output = CodebaseRefactorer.refactor_snippet(source_code, language, graph)
    assert output == "Piranha is probably in an infinite loop."
