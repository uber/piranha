
from polyglot_piranha import Rule


update_enclosing_var_declaration_scala = Rule(
    name="update_enclosing_var_declaration_scala",
    query="cs val :[conf_var] = :[rhs]",
    replace_node="rhs",
    replace="@rhs.getOrCreate()",
    is_seed_rule=False,
    groups={"update_enclosing_var_declaration"},
)

update_spark_context_scala = Rule(
    name="update_spark_context_scala",
    query="cs new SparkContext(@conf_var)",
    replace_node="*",
    replace="@conf_var.sparkContext",
    holes={"conf_var"},
    is_seed_rule=False,
    groups={"update_spark_context"}
)


RULES = [update_enclosing_var_declaration_scala, update_spark_context_scala]
