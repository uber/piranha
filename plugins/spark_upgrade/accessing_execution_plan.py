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

class AccessingExecutionPlan(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={"queryExec": "queryExecution", "execPlan": "executedPlan"},
            language="scala",
        )

    def step_name(self) -> str:
        return "Access execution plans"

    def get_rules(self) -> List[Rule]:
        transform_IDFModel_args = Rule(
            name="accessing_execution_plan",
            query="cs :[dataframe].queryExecution.executedPlan",
            replace_node="*",
            replace="@dataframe.queryExecution.executedPlan.asInstanceOf[AdaptiveSparkPlanExec].initialPlan",
            holes={"queryExec", "execPlan"},
            filters={Filter(
                enclosing_node="(var_definition) @var_def",
                not_contains=["""(
                    (field_expression
                        field: (identifier) @field_id
                        (#eq? @field_id "initialPlan")
                    ) @field_expr
                )"""],
            )}
        )
        return [transform_IDFModel_args]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
