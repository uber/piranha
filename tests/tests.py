# Copyright (c) 2022 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.



from polyglot_piranha import Constraint, execute_piranha, PiranhaArguments, PiranhaOutputSummary, Rule, RuleGraph, OutgoingEdges
from os.path import join, basename
from os import listdir
import re


def test_piranha_rewrite():
    args = PiranhaArguments(
        "test-resources/java/feature_flag_system_1/treated/input",
        "test-resources/java/feature_flag_system_1/treated/configurations",
        "java",
        {
            "stale_flag_name": "STALE_FLAG",
            "treated": "true",
            "treated_complement": "false",
        },
        dry_run=True,
    )

    output_summaries = execute_piranha(args)
    
    assert len(output_summaries) == 2
    expected_paths = [
        "test-resources/java/feature_flag_system_1/treated/input/XPFlagCleanerPositiveCases.java",
        "test-resources/java/feature_flag_system_1/treated/input/TestEnum.java",
    ]
    assert all([o.path in expected_paths for o in output_summaries])
    summary: PiranhaOutputSummary
    for summary in output_summaries:
        assert _is_readable(str(summary))
        for rewrite in summary.rewrites:
            assert _is_readable(str(rewrite)) and _is_readable(str(rewrite.p_match))
            assert rewrite.matched_rule
            assert rewrite.p_match.matched_string and rewrite.p_match.matches

    assert is_as_expected(
        "test-resources/java/feature_flag_system_1/treated", output_summaries
    )

def test_piranha_match_only():
    args = PiranhaArguments(
        "test-resources/java/structural_find/input",
        "test-resources/java/structural_find/configurations",
        "java",
        {},
        dry_run=True,
    )
    output_summaries = execute_piranha(args)
    assert len(output_summaries[0].matches) == 20
    for summary in output_summaries:
        assert _is_readable(str(summary))
        for rule, match in summary.matches:
            assert rule 
            assert _is_readable(str(match))


def test_insert_field_add_import():
    add_field_declaration = Rule (
        name= "add_field_declaration",
        query= 
        """(
            (class_declaration name: (_)@class_name 
                body : (class_body ((_)*) @class_members)  @class_body
            ) @class_declaration
            (#eq? @class_name "FooBar")
        )
        """,
        replace_node="class_body",
        replace="""{
  private List<String> names;\n @class_members
}""",
        constraints= set([
            Constraint(
                matcher= "(class_declaration ) @c_cd",
                queries = ["""
                (
                    (field_declaration (variable_declarator name:(_) @name )) @field
                    (#eq? @name "names")
                )"""]
            )
        ]),
    )

    add_import = Rule (
        name= "add_import",
        query= "(package_declaration) @pkg_dcl",
        replace_node="pkg_dcl",
        replace="""@pkg_dcl
import java.util.List;
""",
        groups= set(["Cleanup Rule"]),
        constraints= set([
            Constraint(
                matcher= "(program ) @prgrm",
                queries = ["""
                (
                    (import_declaration (scoped_identifier) @type ) @import
                    (#eq? @type "java.util.List")
                )"""]
            )
        ]),
    )

    edge1 = OutgoingEdges(
            "add_field_declaration",
            ["add_import"],
            "File"
        )

    rule_graph = RuleGraph(
        rules= [add_field_declaration, add_import],
        edges = [edge1]
        )

    args = PiranhaArguments(
        "test-resources/java/insert_field_and_import/input",
        "test-resources/java/insert_field_and_initializer/configurations",
        "java",
        {
            "stale_flag_name": "STALE_FLAG",
            "treated": "true",
            "treated_complement": "false",
        },
        rule_graph = rule_graph,
        dry_run=True,
    )

    output_summaries = execute_piranha(args)
    assert is_as_expected(
        "test-resources/java/insert_field_and_initializer/", output_summaries
    )

def is_as_expected(path_to_scenario, output_summary):
    expected_output = join(path_to_scenario, "expected")
    for file_name in listdir(expected_output):
        with open(join(expected_output, file_name), "r") as f:
            file_content = f.read()
            expected_content = "".join(file_content.split())
            updated_content = [
                "".join(o.content.split())
                for o in output_summary
                if basename(o.path) == file_name
            ][0]
            if expected_content != updated_content:
                return False
    return True


def _is_readable(input_str: str) -> bool:
    """Check if the input string does not look like :
    `<builtin.PiranhaSummary object at 0x789>`

    Args:
        input_str (str): 
            Input string

    Returns:
        bool: is human readable
    """
    return not any(re.findall(r"\<(.*) object at (.*)\>", input_str))
