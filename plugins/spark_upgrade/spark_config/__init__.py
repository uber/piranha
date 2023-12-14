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
        update_spark_conf_init = Rule(
            name="update_spark_conf_init",
            query="cs new SparkConf()",
            replace_node="*",
            replace='new SparkConf().set("spark.sql.legacy.timeParserPolicy","LEGACY").set("spark.sql.legacy.allowUntypedScalaUDF", "true")',
            filters={
                Filter(
                    not_enclosing_node='cs new SparkConf().set("spark.sql.legacy.timeParserPolicy","LEGACY").set("spark.sql.legacy.allowUntypedScalaUDF", "true")'
                )
            },
        )

        update_spark_session_builder_init = Rule(
            name="update_spark_conf_init",
            query="cs SparkSession.builder()",
            replace_node="*",
            replace='SparkSession.builder().config("spark.sql.legacy.timeParserPolicy","LEGACY").config("spark.sql.legacy.allowUntypedScalaUDF", "true")',
            filters={
                Filter(
                    not_enclosing_node='cs SparkSession.builder().config("spark.sql.legacy.timeParserPolicy","LEGACY").config("spark.sql.legacy.allowUntypedScalaUDF", "true")'
                )
            },
        )

        update_import_array_queue = Rule(
            name="update_import_array_queue",
            query=(
                "cs import org.spark_project.jetty.util.ArrayQueue;"
                if self.language == "java"
                else "cs import org.spark_project.jetty.util.ArrayQueue"
            ),
            replace_node="*",
            replace=(
                "import java.util.ArrayDeque;"
                if self.language == "java"
                else "import java.util.ArrayDeque"
            ),
        )

        return [
            update_spark_conf_init,
            update_spark_session_builder_init,
            update_import_array_queue,
        ]

    def get_edges(self) -> List[OutgoingEdges]:
        return []

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
