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
    OutgoingEdges,
    Rule,
)


class SparkConfigChange(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={
                "spark_conf": "SparkConf",
            },
            language="scala",
        )

    def step_name(self) -> str:
        return "Spark Config Change"

    def get_rules(self) -> List[Rule]:
        # Rule to transform EntropyCalculator() arguments
        spark_conf_change = Rule(
            name="spark_conf_change",
            query="cs new SparkConf()",
            replace_node="*",
            replace='new SparkSession.builder().config("spark.sql.legacy.allowUntypedScalaUDF", "true")',
            holes={"spark_conf"},
        )

        app_name_change = Rule(
            name="app_name_change",
            query="cs :[r].setAppName(:[app_name])",
            replace_node="*",
            replace="@r.appName(@app_name)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        master_name_change = Rule(
            name="master_name_change",
            query="cs :[r].setMaster(:[master])",
            replace_node="*",
            replace="@r.master(@master)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        setter_name_change = Rule(
            name="setter_name_change",
            query="cs :[r].set(:[a1],:[a2])",
            replace_node="*",
            replace="@r.config(@a1, @a2)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        set_all_change = Rule(
            name="set_all_change",
            query="cs :[r].setAll(:[a1])",
            replace_node="*",
            replace="@r.all(@a1)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        set_if_missing = Rule(
            name="set_if_missing",
            query="cs :[r].setIfMissing(:[a1], :[a2])",
            replace_node="*",
            replace="@r.ifMissing(@a1)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        setJars_change = Rule(
            name="setJars_change",
            query="cs :[r].setJars(:[a1])",
            replace_node="*",
            replace="@r.jars(@a1)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        set_executor_env_change_1 = Rule(
            name="set_executor_env_change_1",
            query="cs :[r].setExecutorEnv(:[a1])",
            replace_node="*",
            replace="@r.executorEnv(@a1)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        set_executor_env_change_2 = Rule(
            name="set_executor_env_change_2",
            query="cs :[r].setExecutorEnv(:[a1], :[a2])",
            replace_node="*",
            replace="@r.executorEnv(@a1, @a2)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        set_spark_home_change = Rule(
            name="set_spark_home_change",
            query="cs :[r].setSparkHome(:[a1])",
            replace_node="*",
            replace="@r.sparkHome(@a1)",
            groups={"BuilderPattern"},
            is_seed_rule=False,
        )

        enclosing_var_declaration = Rule(
            name="enclosing_var_declaration",
            query="cs val :[conf_var] = :[rhs]",
            replace_node="rhs",
            replace="@rhs.getOrCreate()",
            is_seed_rule=False,
        )

        update_spark_context = Rule(
            name="update_spark_context",
            query="cs new SparkContext(@conf_var)",
            replace_node="*",
            replace="@conf_var.sparkContext",
            holes={"conf_var"},
            is_seed_rule=False,
        )

        dummy = Rule(name="dummy", is_seed_rule=False)

        return [
            spark_conf_change,
            dummy,
            app_name_change,
            master_name_change,
            setter_name_change,
            set_all_change,
            set_if_missing,
            setJars_change,
            set_executor_env_change_1,
            set_executor_env_change_2,
            set_spark_home_change,
            enclosing_var_declaration,
            update_spark_context,
        ]

    def get_edges(self) -> List[OutgoingEdges]:
        return [
            OutgoingEdges("spark_conf_change", ["dummy"], scope="Parent"),
            OutgoingEdges(
                "dummy",
                [
                    "BuilderPattern",
                    "enclosing_var_declaration",
                ],
                scope="Parent",
            ),
            OutgoingEdges("BuilderPattern", ["dummy"], scope="Parent"),
            OutgoingEdges(
                "enclosing_var_declaration",
                ["update_spark_context"],
                scope="File",
            ),
        ]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
