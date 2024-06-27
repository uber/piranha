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
)


class CalculatorSignatureChange(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={
                "entropy_calculator": "EntropyCalculator",
                "gini_calculator": "GiniCalculator",
                "variance_calculator": "VarianceCalculator",
            },
            language="scala",
        )

    def step_name(self) -> str:
        return "Calculator Signature Change"

    def get_rules(self) -> List[Rule]:
        # Rule to transform EntropyCalculator() arguments
        transform_EntropyCalculator_args = Rule(
            name="transform_EntropyCalculator_args",
            query="cs EntropyCalculator(:[stats])",
            replace_node="*",
            replace="EntropyCalculator(:[stats], :[stats].sum.toLong)",
            holes={"entropy_calculator"},
        )

        # Rule to transform GiniCalculator() arguments
        transform_GiniCalculator_args = Rule(
            name="transform_GiniCalculator_args",
            query="cs GiniCalculator(:[stats])",
            replace_node="*",
            replace="GiniCalculator(:[stats], :[stats].sum.toLong)",
            holes={"gini_calculator"},
        )

        transform_VarianceCalculator_args = Rule(
            name="transform_VarianceCalculator_args",
            query="cs VarianceCalculator(:[stats])",
            replace_node="*",
            replace="VarianceCalculator(:[stats], :[stats].sum.toLong)",
            holes={"variance_calculator"},
        )
        return [
            transform_VarianceCalculator_args,
            transform_GiniCalculator_args,
            transform_EntropyCalculator_args,
        ]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
