# Copyright (c) 2024 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

from __future__ import annotations
from typing import Any, List, Dict
from execute_piranha import ExecutePiranha

from polyglot_piranha import (
    execute_piranha,
    Filter,
    OutgoingEdges,
    Rule,
    PiranhaOutputSummary,
    Match,
    PiranhaArguments,
    RuleGraph,
)

_JAVASPARKCONTEXT_OCE_QUERY = """(
    (object_creation_expression
        type: (_) @oce_typ
    (#eq? @oce_typ "JavaSparkContext")
    ) @oce
)"""

_NEW_SPARK_CONF_CHAIN_QUERY = """(
    (argument_list
        .
        (method_invocation) @mi
        .
        (#match? @mi "^new SparkConf()\\.")
    )
)"""  # matches a chain of method invocations starting with `new SparkConf().`; the chain is the only argument of an argument_list (indicated by the surrounding anchors `.`).

# Note that we don't remove the unused `SparkConf` import; that will be automated somewhere else.
_ADD_IMPORT_RULE = Rule(
    name="add_import_rule",
    query="""(
        (program
            (import_declaration) @imp_decl
        )
    )""",  # matches the last import
    replace_node="imp_decl",
    replace="@imp_decl\nimport org.apache.spark.sql.SparkSession;",
    is_seed_rule=False,
    filters={
        Filter(  # avoids infinite loop
            enclosing_node="((program) @unit)",
            not_contains=[("cs import org.apache.spark.sql.SparkSession;")],
        ),
    },
)


class JavaSparkContextChange(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str], language: str = "java"):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={
                "spark_conf": "SparkConf",
            },
            language=language,
        )

    def __call__(self) -> dict[str, bool]:
        if self.language != "java":
            return {}

        piranha_args = self.get_piranha_arguments()
        summaries: list[PiranhaOutputSummary] = execute_piranha(piranha_args)
        assert summaries is not None

        for summary in summaries:
            file_path: str = summary.path
            match: tuple[str, Match]
            for match in summary.matches:
                if match[0] == "java_match_rule":
                    matched_str = match[1].matched_string

                    replace_str = matched_str.replace(
                        "new SparkConf()",
                        'SparkSession.builder().config("spark.sql.legacy.allowUntypedScalaUDF", "true")',
                    )
                    replace_str = replace_str.replace(".setAppName(", ".appName(")
                    replace_str = replace_str.replace(".setMaster(", ".master(")
                    replace_str = replace_str.replace(".set(", ".config(")
                    replace_str += ".getOrCreate().sparkContext()"

                    # assumes that there's only one match on the file
                    rewrite_rule = Rule(
                        name="rewrite_rule",
                        query=_NEW_SPARK_CONF_CHAIN_QUERY,
                        replace_node="mi",
                        replace=replace_str,
                        filters={
                            Filter(enclosing_node=_JAVASPARKCONTEXT_OCE_QUERY),
                        },
                    )

                    rule_graph = RuleGraph(
                        rules=[rewrite_rule, _ADD_IMPORT_RULE],
                        edges=[
                            OutgoingEdges(
                                "rewrite_rule",
                                to=["add_import_rule"],
                                scope="File",
                            )
                        ],
                    )
                    execute_piranha(
                        PiranhaArguments(
                            language=self.language,
                            rule_graph=rule_graph,
                            paths_to_codebase=[file_path],
                        )
                    )

        if not summaries:
            return {self.step_name(): False}

        return {self.step_name(): True}

    def step_name(self) -> str:
        return "JavaSparkContext Change"

    def get_rules(self) -> List[Rule]:
        if self.language != "java":
            return []

        java_match_rule = Rule(
            name="java_match_rule",
            query=_NEW_SPARK_CONF_CHAIN_QUERY,
            filters={
                Filter(enclosing_node=_JAVASPARKCONTEXT_OCE_QUERY),
            },
        )

        return [java_match_rule]

    def get_edges(self) -> List[OutgoingEdges]:
        return []

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
