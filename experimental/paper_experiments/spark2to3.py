from typing import Any, Dict, Optional, Tuple
from tree_sitter import Node, Tree
from utils import parse_code, traverse_tree, rewrite, SOURCE_CODE


relevant_builder_method_names_mapping = {
    "setAppName": "appName",
    "setMaster": "master",
    "set": "config",
    "setAll": "all",
    "setIfMissing": "ifMissing",
    "setJars": "jars",
    "setExecutorEnv": "executorEnv",
    "setSparkHome":  "sparkHome",
}


def get_initializer_named(tree: Tree, name: str):
    for node in traverse_tree(tree):
        if node.type == "object_creation_expression":
            oce_type = node.child_by_field_name("type")
            if oce_type and oce_type.text.decode() == name:
                return node


def get_enclosing_variable_declaration_name_type(
    node: Node,
) -> Tuple[Node | None, str | None, str | None]:
    name, typ, nd = None, None, None
    if node.parent and node.parent.type == "variable_declarator":
        n = node.parent.child_by_field_name("name")
        if n:
            name = n.text.decode()
        if (
            node.parent.parent
            and node.parent.parent.type == "local_variable_declaration"
        ):
            t = node.parent.parent.child_by_field_name("type")
            if t:
                typ = t.text.decode()
                nd = node.parent.parent
    return nd, name, typ


def all_enclosing_method_invocations(node: Node) -> list[Node]:
    if node.parent and node.parent.type == "method_invocation":
        return [node.parent] + all_enclosing_method_invocations(node.parent)
    else:
        return []


def build_spark_session_builder(builder_mappings: list[tuple[str, Node]]):
    replacement_expr = 'new SparkSession.builder().config("spark.sql.legacy.allowUntypedScalaUDF", "true")'
    for name, args in builder_mappings:
        replacement_expr += f".{name}{args.text.decode()}"
    return replacement_expr


def update_spark_conf_init(
    tree: Tree, src_code: str, state: Dict[str, Any]
) -> Tuple[Tree, str]:
    spark_conf_init = get_initializer_named(tree, "SparkConf")
    if not spark_conf_init:
        print("No SparkConf initializer found")
        return tree, src_code

    encapsulating_method_invocations = all_enclosing_method_invocations(
        spark_conf_init
    )
    builder_mappings = []
    for n in encapsulating_method_invocations:
        name = n.child_by_field_name("name")
        if (
            name
            and name.text.decode()
            in relevant_builder_method_names_mapping.keys()
        ):
            builder_mappings.append(
                (
                    relevant_builder_method_names_mapping[name.text.decode()],
                    n.child_by_field_name("arguments"),
                )
            )

    builder_mapping = build_spark_session_builder(builder_mappings)

    outermost_node_builder_pattern = (
        encapsulating_method_invocations[-1]
        if encapsulating_method_invocations
        else spark_conf_init
    )

    node, name, typ = get_enclosing_variable_declaration_name_type(
        outermost_node_builder_pattern
    )

    if not (node and name and typ):
        print("Not in a variable declaration")
        return tree, src_code

    declaration_replacement = (
        f"SparkSession {name} = {builder_mapping}.getOrCreate();"
    )

    state["spark_conf_name"] = name

    return rewrite(node, src_code, declaration_replacement)


def update_spark_context_init(
    tree: Tree, source_code: str, state: Dict[str, Any]
):
    if "spark_conf_name" not in state:
        print("Needs the name of the variable holding the SparkConf")
        return tree, source_code
    spark_conf_name = state["spark_conf_name"]
    init = get_initializer_named(tree, "JavaSparkContext")
    if not init:
        return tree, source_code

    node, name, typ = get_enclosing_variable_declaration_name_type(init)
    if node:
        return rewrite(
            node,
            source_code,
            f"SparkContext {name} = {spark_conf_name}.sparkContext()",
        )
    else:
        return rewrite(init, source_code, f"{spark_conf_name}.sparkContext()")


def get_setter_call(variable_name: str, tree: Tree) -> Optional[Node]:
    for node in traverse_tree(tree):
        if node.type == "method_invocation":
            name = node.child_by_field_name("name")
            r = node.child_by_field_name("object")
            if name and r: 
                name = name.text.decode()
                r = r.text.decode()
                if r == variable_name and name in relevant_builder_method_names_mapping.keys():
                    return node


def update_spark_conf_setters(
    tree: Tree, source_code: str, state: Dict[str, Any]
):
    setter_call =   get_setter_call(state["spark_conf_name"], tree)
    if setter_call:
        rcvr = state["spark_conf_name"]
        invc = setter_call.child_by_field_name("name")
        args = setter_call.child_by_field_name("arguments")
        if rcvr and invc and args:
            new_fn = relevant_builder_method_names_mapping[invc.text.decode()]
            replacement =  f"{rcvr}.{new_fn}{args.text.decode()}"
            return rewrite(setter_call, source_code, replacement)
    return tree, source_code

state = {}
no_change = False
while not no_change:
    TREE: Tree = parse_code("java", SOURCE_CODE)
    original_code = SOURCE_CODE
    TREE, SOURCE_CODE = update_spark_conf_init(TREE, SOURCE_CODE, state)
    TREE, SOURCE_CODE = update_spark_context_init(TREE, SOURCE_CODE, state)
    no_change = SOURCE_CODE == original_code
    no_setter_found = False
    while not no_setter_found:
        b4_code = SOURCE_CODE
        TREE, SOURCE_CODE = update_spark_conf_setters(TREE, SOURCE_CODE, state)
        no_setter_found = SOURCE_CODE == b4_code
