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

_JAVASPARKCONTEXT_OCE_QUERY = """(
    (object_creation_expression
        type: (_) @oce_typ
    (#eq? @oce_typ "JavaSparkContext")
    ) @oce
)"""

_SPARK_SESSION_BUILDER_CHAIN_QUERY = """(
    (method_invocation
        object: (method_invocation
            object: (identifier) @spark_session
            name: (identifier) @receiver
        )
        (#eq? @spark_session "SparkSession")
        (#eq? @receiver "builder")
    ) @mi
)"""

_EXPR_STMT_CHAIN_ENDS_WITH_GETORCREATE_QUERY = """(
    (expression_statement
    	(method_invocation
        	name: (identifier) @last
        )
        (#match? @last "getOrCreate")
    ) @expr_stmt
)"""

_SCALA_CHAIN_ENDS_WITH_GETORCREATE_QUERY = """(
    (field_expression
        field: (identifier) @last_field
        (#eq? @last_field "getOrCreate")
    ) @field_expr
)"""

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
        # filters cannot be added without reinstantiating Rule(), so we create the full filter set before
        fs = {
            Filter(
                not_enclosing_node='cs new SparkConf().set("spark.sql.legacy.timeParserPolicy","LEGACY").set("spark.sql.legacy.allowUntypedScalaUDF", "true")'
            ),
        }
        if self.language == "java":
            fs.add(Filter(not_enclosing_node=_JAVASPARKCONTEXT_OCE_QUERY))
            fs.add(
                Filter(not_enclosing_node=_EXPR_STMT_CHAIN_ENDS_WITH_GETORCREATE_QUERY)
            )
        elif self.language == "scala":
            fs.add(Filter(not_enclosing_node=_SCALA_CHAIN_ENDS_WITH_GETORCREATE_QUERY))

        update_spark_conf_init = Rule(
            name="update_spark_conf_init",
            query="cs new SparkConf()",
            replace_node="*",
            replace='new SparkConf().set("spark.sql.legacy.timeParserPolicy","LEGACY").set("spark.sql.legacy.allowUntypedScalaUDF", "true")',
            filters=fs,
        )

        fs2 = {
            Filter(
                not_enclosing_node='cs SparkSession.builder().config("spark.sql.legacy.timeParserPolicy","LEGACY").config("spark.sql.legacy.allowUntypedScalaUDF", "true")'
            )
        }
        if self.language == "java":
            fs2.add(Filter(not_enclosing_node=_JAVASPARKCONTEXT_OCE_QUERY))
            fs2.add(Filter(not_enclosing_node=_SPARK_SESSION_BUILDER_CHAIN_QUERY))
            fs2.add(
                Filter(not_enclosing_node=_EXPR_STMT_CHAIN_ENDS_WITH_GETORCREATE_QUERY)
            )
        elif self.language == "scala":
            fs2.add(Filter(not_enclosing_node=_SCALA_CHAIN_ENDS_WITH_GETORCREATE_QUERY))

        update_spark_session_builder_init = Rule(
            name="update_spark_conf_init",
            query="cs SparkSession.builder()",
            replace_node="*",
            replace='SparkSession.builder().config("spark.sql.legacy.timeParserPolicy","LEGACY").config("spark.sql.legacy.allowUntypedScalaUDF", "true")',
            filters=fs2,
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
