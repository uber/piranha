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
from pathlib import Path
from polyglot_piranha import Filter, execute_piranha, PiranhaArguments, PiranhaOutputSummary, Rule, RuleGraph, OutgoingEdges
from os.path import join, basename
from os import listdir
import re
import pytest

def test_piranha_rewrite():
    args = PiranhaArguments(
        path_to_configurations="test-resources/java/feature_flag_system_1/treated/configurations",
        language="java",
        substitutions={
            "stale_flag_name": "STALE_FLAG",
            "treated": "true",
            "treated_complement": "false",
        },
        paths_to_codebase=["test-resources/java/feature_flag_system_1/treated/input"],
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

def test_piranha_rewrite_custom():
    r1 = Rule(
        name="replace_method",
        query=f"""(
            (method_declaration
                (modifiers) @modifier
                name: (_) @name
                body: (block) @body
            ) @method
            (#match? @name "empty")
            (#match? @modifier "private")
        )""",
        replace_node="method",
        replace="",
        is_seed_rule=True,
    )
    edge_1 = OutgoingEdges("replace_method", to=["delete_usages"], scope="Class")
    args = PiranhaArguments(
        language="java",
        paths_to_codebase=["test-resources/java/custom_rules/input"],
        path_to_custom_builtin_rules="test-resources/java/custom_rules/custom_rules",
        rule_graph=RuleGraph(rules=[r1], edges=[edge_1]),
        dry_run=True,
    )

    output_summaries = execute_piranha(args)

    assert len(output_summaries) == 1
    expected_paths = [
        "test-resources/java/custom_rules/input/Sample.java",
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
        "test-resources/java/custom_rules/", output_summaries
    )



def test_piranha_match_only():
    args = PiranhaArguments(
        path_to_configurations="test-resources/java/structural_find_with_include_exclude/configurations",
        language="java",
        paths_to_codebase=["test-resources/java/structural_find_with_include_exclude/input"],
        dry_run=True,
        exclude=["*/folder_2_1/**/*"]
    )
    output_summaries = execute_piranha(args)
    assert sum([len(summary.matches) for summary in output_summaries]) == 44
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
        filters= set([
            Filter(
                not_contains = ["""(
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
        filters= set([
            Filter(
                enclosing_node= "(program ) @prgrm",
                not_contains = ["""
                (
                    (import_declaration (scoped_identifier) @type ) @import
                    (#eq? @type "java.util.List")
                )"""]
            )
        ]),
        is_seed_rule= False
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
        paths_to_codebase=["test-resources/java/insert_field_and_import/input"],
        language="java",
        rule_graph = rule_graph,
        dry_run=True,
    )

    output_summaries = execute_piranha(args)
    assert is_as_expected(
        "test-resources/java/insert_field_and_initializer/", output_summaries
    )

def test_delete_unused_field():

    delete_unused_field = Rule (
        name= "delete_unused_field",
        query=
        """(
        ((field_declaration
            declarator: (_) @id_name) @decl)
        (#match? @decl "^private")
        )
        """,
        replace_node="decl",
        replace="",
        filters= set([
            Filter(
                enclosing_node= "(class_declaration ) @c_cd",
                contains = """(
                    (identifier) @name
                    (#eq? @name "@id_name")
                )""",
                at_most= 1
            )
        ]),
    )

    rule_graph = RuleGraph(
        rules= [delete_unused_field],
        edges = []
    )

    args = PiranhaArguments(
        paths_to_codebase=["test-resources/java/delete_unused_field/input"],
        language="java",
        rule_graph = rule_graph,
        dry_run=True,
    )

    output_summaries = execute_piranha(args)
    print(output_summaries[0].content)
    assert is_as_expected(
        "test-resources/java/delete_unused_field/", output_summaries
    )


def test_incorrect_import():
    delete_unused_field = Rule (
        name= "delete_unused_field",
        query=
        """(
        (import_declaration @import_decl
        )
        """,
    )


    with pytest.raises(BaseException, match = "Incorrect Rule Graph - Cannot parse") as e:
        rule_graph = RuleGraph(
            rules= [delete_unused_field],
            edges = []
            )

def is_as_expected(path_to_scenario, output_summary):
    expected_output = join(path_to_scenario, "expected")
    input_dir = join(path_to_scenario, "input")
    for file_name in listdir(expected_output):
        with open(join(expected_output, file_name), "r") as f:
            file_content = f.read()
            expected_content = "".join(file_content.split())

            # Search for the file in the output summary
            updated_content = [
                "".join(o.content.split())
                for o in output_summary
                if basename(o.path) == file_name
            ]
            # Check if the file was rewritten
            if updated_content:
                if expected_content != updated_content[0]:
                    return False
            else:
                # The scenario where the file is not expected to be rewritten
                original_content= Path(join(input_dir, file_name)).read_text()
                if expected_content != "".join(original_content.split()):
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

def test_java_toplevel_method_decl():
    """method_declaration for Java should be accepted as top level node.
    In other words, a code_snippet consisting of just a method, with no `public class A {}`
    should not raise an error.
    Possible from tree-sitter-java >= v0.20.2 (https://github.com/tree-sitter/tree-sitter-java/pull/160).
    """

    _java_toplevel_mdecl_matches_anything("public static void main(String args) {  }")
    _java_toplevel_mdecl_matches_anything("public void foo() {  }")
    _java_toplevel_mdecl_matches_anything("void foo() {  }")
    _java_toplevel_mdecl_matches_anything("private void foo() {  }")

def _java_toplevel_mdecl_matches_anything(code_snippet: str) -> bool:
        match_anything_rule = Rule(name="Match anything", query="cs :[x]")
        args = PiranhaArguments(
            code_snippet=code_snippet,
            language="java",
            rule_graph=RuleGraph(rules=[match_anything_rule], edges=[]),
            dry_run=False,
            allow_dirty_ast=False,
        )
        try:
            summaries = execute_piranha(args)
            return len(summaries) > 0
        except:
            assert False, f"Java method_declaration as top level node should not raise an error:\n{code_snippet}"
