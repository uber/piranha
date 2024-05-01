# Copyright (c) 2023 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

from typing import Any, List, Dict
from execute_piranha import ExecutePiranha

from polyglot_piranha import (
    Rule,
    Filter,
)

_INSTANCE_EXPR_QUERY = """(
    (instance_expression
        (type_identifier) @typ_id
        arguments: (arguments
            (_)
            (_)
            (_)
        )
        (#eq? @typ_id "Instance")
    ) @inst
)"""


class GradientBoostTrees(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={"gbt": "GradientBoostedTrees"},
            language="scala",
        )

    def step_name(self) -> str:
        return "Access execution plans"

    def get_rules(self) -> List[Rule]:
        gradient_boost_trees = Rule(
            name="transform_GradientBoostedTrees_run",
            query="""cs GradientBoostedTrees.run(
                :[dataset],
                :[strategy],
                :[seed],
                :[featureSubsetStrategy]
            )""",
            replace_node="*",
            replace="""GradientBoostedTrees.run(
                @dataset.map(data => new Instance(data.label, 1.0, data.features)),
                @strategy,
                @seed,
                @featureSubsetStrategy
            )""",
            filters={
                Filter(
                    not_contains=[_INSTANCE_EXPR_QUERY],
                )
            },
            holes={"gbt"},
        )

        gradient_boost_trees_comment = Rule(
            name="transform_GradientBoostedTrees_run_comment",
            query="""cs GradientBoostedTrees.run(
                :[dataset],
                :[strategy],
                :[seed],
                :[featureSubsetStrategy] :[comment]
            )""",
            replace_node="*",
            replace="""GradientBoostedTrees.run(
                @dataset.map(data => new Instance(data.label, 1.0, data.features)),
                @strategy,
                @seed,
                @featureSubsetStrategy
            )""",
            filters={
                Filter(
                    not_contains=[_INSTANCE_EXPR_QUERY],
                )
            },
            holes={"gbt"},
        )
        return [gradient_boost_trees, gradient_boost_trees_comment]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
