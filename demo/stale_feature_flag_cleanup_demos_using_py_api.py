from os.path import join, dirname, getmtime, exists
from polyglot_piranha import Rule, RuleGraph, execute_piranha, PiranhaArguments
import logging
from logging import info

feature_flag_dir = join(dirname(__file__), "feature_flag_cleanup_using_py_api")


def run_java_ff_demo():
    info("Running the stale feature flag cleanup demo for Java")

    directory_path = join(feature_flag_dir, "java")
    modified_file_path = join(directory_path, "SampleClass.java")

    update_is_treated = Rule(
        name="update_is_treated",
        # :[e] matches any node (its like a hole) and @flag_name refers to the substitution
        query='cs :[e].isTreated("@flag_name")',
        replace_node="*",
        # @treated refers to the substitution
        replace="@treated",
        groups={"replace_expression_with_boolean_literal"},
        # This is a list of holes that need to be filled in for the rule to be applied
        holes={"treated", "flag_name"}
    )

    args = PiranhaArguments(
        "java",
        paths_to_codebase=[directory_path],
        substitutions={"flag_name": "SAMPLE_STALE_FLAG", "treated": "true"},
        rule_graph=RuleGraph(rules=[update_is_treated], edges=[])
    )

    output_summaries = execute_piranha(args)

    old_mtime = getmtime(modified_file_path)

    assert len(output_summaries) == 1

    for summary in output_summaries:
        assert len(summary.rewrites) > 0

    new_mtime = getmtime(modified_file_path)

    assert old_mtime < new_mtime


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

run_java_ff_demo()
print("Completed running the stale feature flag cleanup demos")
