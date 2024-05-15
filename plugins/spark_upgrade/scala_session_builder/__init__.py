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
    Edit,
    PiranhaArguments,
    RuleGraph,
)

VAL_DEF_QUERY = """(
    (val_definition
        pattern: (identifier) @val_id
        type: (type_identifier) @type_id
        value: (call_expression
            function: (identifier) @func_call
        )
        (#eq? @type_id "SparkSession")
        (#eq? @func_call "spy")
    ) @val_def
)"""

QUERY = f"""(
	(function_definition
      body: (block
        {VAL_DEF_QUERY}
        .
        (call_expression
        	function: (field_expression
            	value: (field_expression
                	value: (identifier) @lhs
                    field: (identifier) @rhs
                )
                field: (identifier) @call_name
                (#eq? @lhs @val_id)
                (#eq? @rhs "sqlContext")
                (#eq? @call_name "setConf")
            )
        )+ @calls
      )
    ) @func_def
)"""


class ScalaSessionBuilder(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str], language: str = "scala"):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={},
            language=language,
        )

    def __call__(self) -> dict[str, bool]:
        if self.language != "scala":
            return {}

        piranha_args = self.get_piranha_arguments()
        summaries: list[PiranhaOutputSummary] = execute_piranha(piranha_args)
        assert summaries is not None

        for summary in summaries:
            file_path: str = summary.path
            edit: Edit
            if len(summary.rewrites) == 0:
                continue

            print(f"rewrites: {len(summary.rewrites)}")

            calls_to_add_str = ""
            # the rewrite's edit will have `calls` with all matches
            edit = summary.rewrites[0]
            if edit.matched_rule == "delete_calls_query":
                match: Match = edit.p_match
                val_id = match.matches["val_id"]
                calls = match.matches["calls"]
                print(f"calls: {calls}")
                calls_to_add_str = calls.replace(
                    f"{val_id}.sqlContext.setConf", ".config"
                )

            match = summary.rewrites[0].p_match
            val_def = match.matches["val_def"]

            assert isinstance(val_def, str)
            assert "getOrCreate()" in val_def

            replace_str = calls_to_add_str + "\n.getOrCreate()"
            new_val_def = val_def.replace(".getOrCreate()", replace_str)

            replace_val_def_rule = Rule(
                name="replace_val_def_rule",
                query=VAL_DEF_QUERY,
                replace_node="val_def",
                replace=new_val_def,
                filters={
                    Filter(
                        enclosing_node="(val_definition) @_vl_def",
                        not_contains=(
                            [
                                """(
                                    (identifier) @conf_id
                                    (#eq? @conf_id "config")
                                )"""
                            ]
                        ),
                    )
                },
            )

            rule_graph = RuleGraph(
                rules=[replace_val_def_rule],
                edges=[],
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
        return "Spark spy SessionBuilder"

    def get_rules(self) -> List[Rule]:
        if self.language != "scala":
            return []

        delete_calls_query = Rule(
            name="delete_calls_query",
            query=QUERY,
            replace_node="calls",
            replace="",
        )

        return [delete_calls_query]

    def get_edges(self) -> List[OutgoingEdges]:
        return []

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
