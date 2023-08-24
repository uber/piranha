from collections import Counter
from os.path import join, dirname
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
from logging import info

match_only_dir = join(dirname(__file__), "match_only")


def java_demo():
    info("Running the Match-only demo for Java")
    args = PiranhaArguments(
        "java",
        substitutions={},
        paths_to_codebase=[join(match_only_dir, "java")],
        path_to_configurations=join(match_only_dir, "java/configurations"),
    )
    output_summary_java = execute_piranha(args)

    rule_match_counter = Counter(
        [m[0] for m in output_summary_java[0].matches])

    assert rule_match_counter["find_fooBar_anywhere"] == 2

    assert rule_match_counter["find_barFoo_in_non_static_method"] == 1


def go_demo():
    info("Running the Match-only demo for go")

    args = PiranhaArguments(
        "go",
        substitutions={},
        paths_to_codebase=[join(match_only_dir, "go")],
        path_to_configurations=join(match_only_dir, "go/configurations"),
    )
    output_summary_go = execute_piranha(args)

    rule_match_counter = Counter([m[0] for m in output_summary_go[0].matches])

    assert rule_match_counter["find_go_stmt_for_loop"] == 1

    assert rule_match_counter["find_for"] == 4


def ts_demo():
    info("Running the Match-only demo for TypeScript")

    args = PiranhaArguments(
        "ts",
        substitutions={},
        paths_to_codebase=[join(match_only_dir, "ts")],
        path_to_configurations=join(match_only_dir, "ts/configurations"),
    )
    output_summary_typescript = execute_piranha(args)

    rule_match_counter = Counter(
        [m[0] for m in output_summary_typescript[0].matches])

    assert rule_match_counter["find_fors"] == 3
    assert rule_match_counter["find_fors_within_functions"] == 2
    assert rule_match_counter["find_fors_within_functions_not_within_whiles"] == 1


def tsx_demo():
    info("Running the Match-only demo for TypeScript with React")
    args = PiranhaArguments(
        "tsx",
        substitutions={},
        paths_to_codebase=[join(match_only_dir, "tsx")],
        path_to_configurations=join(match_only_dir, "tsx/configurations"),
    )
    output_summary_typescript = execute_piranha(args)

    rule_match_counter = Counter(
        [m[0] for m in output_summary_typescript[0].matches])

    assert rule_match_counter["find_jsx_elements"] == 4
    assert rule_match_counter["find_props_identifiers_within_b_jsx_elements"] == 2
    assert (
        rule_match_counter[
            "find_props_identifiers_within_variable_declarators_not_within_divs"
        ]
        == 2
    )


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

java_demo()
go_demo()
ts_demo()
tsx_demo()

info("Completed running the Match-only demo")
