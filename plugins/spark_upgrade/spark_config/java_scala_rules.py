from polyglot_piranha import Rule

spark_conf_change_java_scala = Rule(
    name="spark_conf_change_java_scala",
    query="cs new SparkConf()",
    replace_node="*",
    replace='new SparkSession.builder().config("spark.sql.legacy.allowUntypedScalaUDF", "true")',
    holes={"spark_conf"},
)

def get_setter_rules(name: str, query: str, replace: str) -> list[Rule]:
    return [
        Rule(
            name=name,
            query=query.format(receiver=":[r]"),
            replace_node="*",
            replace=replace.format(receiver="@r"),
            groups={"BuilderPattern"},
            is_seed_rule=False,
        ),
        Rule(
            name=name + "_stand_alone_call",
            query=query.format(receiver="@conf_var"),
            replace_node="*",
            replace=replace.format(receiver="@conf_var"),
            holes={"conf_var"},
            groups={"StandAloneCall"},
            is_seed_rule=False,
        ),
    ]


RULES =  [spark_conf_change_java_scala] + get_setter_rules(
        "app_name_change_java_scala",
        "cs {receiver}.setAppName(:[app_name])",
        "{receiver}.appName(@app_name)",
    ) + get_setter_rules(
        "master_name_change_java_scala",
        "cs {receiver}.setMaster(:[master])",
        "{receiver}.master(@master)",
    ) + get_setter_rules(
        "setter_name_change_java_scala",
        "cs {receiver}.set(:[a1],:[a2])",
        "{receiver}.config(@a1, @a2)",
    ) + get_setter_rules(
        "set_all_change_java_scala",
        "cs {receiver}.setAll(:[a1])",
        "{receiver}.all(@a1)",
    )  + get_setter_rules(
        "set_if_missing_java_scala",
        "cs {receiver}.setIfMissing(:[a1], :[a2])",
        "{receiver}.ifMissing(@a1)",
    )  + get_setter_rules(
        "set_jars_change_java_scala",
        "cs {receiver}.setJars(:[a1])",
        "{receiver}.jars(@a1)",
    ) + get_setter_rules(
        "set_executor_env_2_change_java_scala",
        "cs {receiver}.setExecutorEnv(:[a1], :[a2])",
        "{receiver}.executorEnv(@a1, @a2)",
    ) + get_setter_rules(
        "set_executor_env_1_change_java_scala",
        "cs {receiver}.setExecutorEnv(:[a1])",
        "{receiver}.executorEnv(@a1)",
    ) + get_setter_rules(
        "set_spark_home_change_java_scala",
        "cs {receiver}.setSparkHome(:[a1])",
        "{receiver}.sparkHome(@a1)",
    ) + [Rule(name="dummy", is_seed_rule=False)]
