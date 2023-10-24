from polyglot_piranha import Filter, OutgoingEdges, Rule


def insert_import(rule_name, fq_type: str) -> Rule:
    return Rule(
        name=rule_name,
        query="(package_declaration) @pkg",
        replace_node="pkg",
        replace=f"@pkg \n import {fq_type};",
        is_seed_rule=False,
        filters={
            Filter(
                enclosing_node="(program) @cu",
                not_contains=[f"cs import {fq_type};"],
            )
        },
    )

update_enclosing_var_declaration_java = Rule(
    name="update_enclosing_var_declaration_java",
    query="cs :[type] :[conf_var] = :[rhs];",
    replace_node="*",
    replace="SparkSession @conf_var = @rhs.getOrCreate();",
    is_seed_rule=False,
    groups={"update_enclosing_var_declaration"},
)

insert_import_spark_session_java = insert_import(
    "insert_import_spark_session_java", "org.apache.spark.sql.SparkSession"
)

update_spark_context_java = Rule(
    name="update_spark_context_java",
    query="cs new JavaSparkContext(@conf_var)",
    replace_node="*",
    replace="@conf_var.sparkContext()",
    holes={"conf_var"},
    is_seed_rule=False,
    groups={"update_spark_context"},
)


update_spark_context_var_decl_lhs_java = Rule(
    name="update_spark_context_var_decl_lhs_java",
    query="cs JavaSparkContext :[v] = :[lhs];",
    replace_node="*",
    replace="SparkContext @v = @lhs;",
    holes={"conf_var"},
    is_seed_rule=False,
    groups={"update_spark_context"},
)

insert_import_spark_context_java = insert_import(
    "insert_import_spark_context_java", "org.apache.spark.SparkContext"
)

RULES = [
    update_enclosing_var_declaration_java,
    insert_import_spark_session_java,
    update_spark_context_java,
    insert_import_spark_context_java,
    update_spark_context_var_decl_lhs_java,
]

EDGES = [
    OutgoingEdges(
        "update_enclosing_var_declaration_java",
        ["insert_import_spark_session_java"],
        scope="File",
    ),
    OutgoingEdges(
        "update_spark_context_java",
        ["update_spark_context_var_decl_lhs_java"],
        scope="Parent",
    ),
    OutgoingEdges(
        "update_spark_context_var_decl_lhs_java",
        ["insert_import_spark_context_java"],
        scope="File",
    ),
]
