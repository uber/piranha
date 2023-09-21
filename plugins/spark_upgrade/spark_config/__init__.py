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
    Filter,
    OutgoingEdges,
    Rule,
)
import spark_config.scala_rules as scala_rules
import spark_config.java_rules as java_rules
import spark_config.java_scala_rules as java_scala_rules


class SparkConfigChange(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str], language: str = "scala"):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={
                "spark_conf": "SparkConf",
            },
            language=language,
        )

    def step_name(self) -> str:
        return "Spark Config Change"

    def get_rules(self) -> List[Rule]:
        return java_scala_rules.RULES + (
            scala_rules.RULES if self.language == "scala" else java_rules.RULES
        )

    def get_edges(self) -> List[OutgoingEdges]:
        return [
            OutgoingEdges(
                "spark_conf_change_java_scala", ["dummy"], scope="ParentIterative"
            ),
            OutgoingEdges("BuilderPattern", ["dummy"], scope="ParentIterative"),
            OutgoingEdges(
                "dummy",
                [
                    "BuilderPattern",
                    "update_enclosing_var_declaration",
                ],
                scope="ParentIterative",
            ),
            OutgoingEdges(
                "update_enclosing_var_declaration",
                ["update_spark_context"],
                scope="File",
            ),
        ] + ([] if self.language == "scala" else java_rules.EDGES)

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
