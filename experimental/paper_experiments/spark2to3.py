from typing import Any, Dict, Optional, Tuple
from tree_sitter import Node, Tree
from utils import (
    JAVA,
    SCALA_SOURCE_CODE,
    parse_code,
    traverse_tree,
    rewrite,
    JAVA_SOURCE_CODE,
    SCALA,
)


relevant_builder_method_names_mapping = {
    "setAppName": "appName",
    "setMaster": "master",
    "set": "config",
    "setAll": "all",
    "setIfMissing": "ifMissing",
    "setJars": "jars",
    "setExecutorEnv": "executorEnv",
    "setSparkHome": "sparkHome",
}


def get_initializer_named(
    tree: Tree, name: str, language: str
) -> Optional[Node]:
    for node in traverse_tree(tree):
        if language == JAVA:
            if node.type == "object_creation_expression":
                oce_type = node.child_by_field_name("type")
                if oce_type and oce_type.text.decode() == name:
                    return node
        if language == SCALA:
            if node.type == "instance_expression":
                if any(c.text.decode() == name for c in node.children):
                    return node


def get_enclosing_variable_declaration_name_type(
    node: Node, language: str
) -> Tuple[Node | None, str | None]:
    name, nd = None, None
    if language == JAVA:
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
                    nd = node.parent.parent
    if language == SCALA:
        if (
            node.parent
            and node.parent.type == "val_definition"
        ):
            n = node.parent.child_by_field_name("pattern")
            if n:
                name = n.text.decode()
                nd = node.parent
    return nd, name


def all_enclosing_method_invocations(node: Node, language: str) -> list[Node]:
    if language == JAVA:
        if node.parent and node.parent.type == "method_invocation":
            return [node.parent] + all_enclosing_method_invocations(
                node.parent, language
            )
        else:
            return []
    else:
        if node.parent and node.parent.parent and node.parent.parent.type == "call_expression":
            return [node.parent.parent] + all_enclosing_method_invocations(
                node.parent.parent, language
            )
        else:
            return []


def build_spark_session_builder(builder_mappings: list[tuple[str, Node]]):
    replacement_expr = 'new SparkSession.builder().config("spark.sql.legacy.allowUntypedScalaUDF", "true")'
    for name, args in builder_mappings:
        replacement_expr += f".{name}{args.text.decode()}"
    return replacement_expr


def update_spark_conf_init(
    tree: Tree, src_code: str, state: Dict[str, Any], language: str
) -> Tuple[Tree, str]:
    spark_conf_init = get_initializer_named(tree, "SparkConf", language)
    if not spark_conf_init:
        print("No SparkConf initializer found")
        return tree, src_code

    encapsulating_method_invocations = all_enclosing_method_invocations(
        spark_conf_init, language
    )
    builder_mappings = []
    for n in encapsulating_method_invocations:
        name = (
            n.child_by_field_name("name")
            if language == JAVA
            else n.children[0].children[2]
        )
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

    node, name = get_enclosing_variable_declaration_name_type(
        outermost_node_builder_pattern, language
    )

    if not (node and name):
        print("Not in a variable declaration")
        return tree, src_code

    declaration_replacement = get_declaration_replacement(
        name, builder_mapping, language
    )

    state["spark_conf_name"] = name

    return rewrite(node, src_code, declaration_replacement, language)


def get_declaration_replacement(name, builder_mapping, language):
    if language == JAVA:
        return f"SparkSession {name} = {builder_mapping}.getOrCreate();"
    else:
        return f"val {name} = {builder_mapping}.getOrCreate()"


def update_spark_context_init(
    tree: Tree, source_code: str, state: Dict[str, Any], language: str
):
    if "spark_conf_name" not in state:
        print("Needs the name of the variable holding the SparkConf")
        return tree, source_code
    spark_conf_name = state["spark_conf_name"]
    init = get_initializer_named(tree, "JavaSparkContext", language)
    if not init:
        return tree, source_code

    node, name = get_enclosing_variable_declaration_name_type(init, language)
    if node:
        return rewrite(
            node,
            source_code,
            f"SparkContext {name} = {spark_conf_name}.sparkContext()",
            language
        )
    else:
        return rewrite(init, source_code, f"{spark_conf_name}.sparkContext()")


def get_setter_call(variable_name: str, tree: Tree, language: str) -> Optional[Node]:
    for node in traverse_tree(tree):
        if language == JAVA:
            if node.type == "method_invocation":
                name = node.child_by_field_name("name")
                r = node.child_by_field_name("object")
                if name and r:
                    name = name.text.decode()
                    r = r.text.decode()
                    if (
                        r == variable_name
                        and name in relevant_builder_method_names_mapping.keys()
                    ):
                        return node
        if language == SCALA:
            if node.type == "call_expression":
                _fn = node.child_by_field_name("function")
                if not _fn:
                    continue
                name = _fn.child_by_field_name("field")
                r = _fn.child_by_field_name("value")
                if name and r:
                    name = name.text.decode()
                    r = r.text.decode()
                    if (
                        r == variable_name
                        and name in relevant_builder_method_names_mapping.keys()
                    ):
                        return node


def update_spark_conf_setters(
    tree: Tree, source_code: str, state: Dict[str, Any], language: str
):
    setter_call = get_setter_call(state["spark_conf_name"], tree, language)
    if setter_call:
        rcvr = state["spark_conf_name"]
        invc = (
            setter_call.child_by_field_name("name")
            if language == JAVA
            else setter_call.children[0].children[2] 
        )
        args = setter_call.child_by_field_name("arguments")
        if rcvr and invc and args:
            new_fn = relevant_builder_method_names_mapping[invc.text.decode()]
            replacement = f"{rcvr}.{new_fn}{args.text.decode()}"
            return rewrite(setter_call, source_code, replacement, language)
    return tree, source_code


def insert_import_statement(
    tree: Tree, source_code: str, import_statement: str, language: str
):
    for import_stmt in traverse_tree(tree):
        if import_stmt.type == "import_declaration":
            if import_stmt.text.decode() == f"import {import_statement}" + (";" if language == JAVA else ""):
                return tree, source_code

    package_decl = [
        n
        for n in traverse_tree(tree)
        if n.type
        == ("package_declaration" if language == JAVA else "package_clause")
    ]
    if not package_decl:
        return tree, source_code
    package_decl = package_decl[0]
    if language == JAVA:
        return rewrite(
            package_decl,
            source_code,
            f"{package_decl.text.decode()}\nimport {import_statement};",
            language
        )
    return rewrite(
        package_decl,
        source_code,
        f"{package_decl.text.decode()}\nimport {import_statement}",
        language
    )


def run(language, source_code):
    state = {}
    no_change = False
    while not no_change:
        TREE: Tree = parse_code(language, source_code)
        original_code = source_code
        TREE, source_code = update_spark_conf_init(
            TREE, source_code, state, language
        )
        TREE, source_code = insert_import_statement(
            TREE, source_code, "org.apache.spark.sql.SparkSession", language
        )
        TREE, source_code = insert_import_statement(
            TREE, source_code, "org.apache.spark.SparkContext", language
        )
        TREE, source_code = update_spark_context_init(
            TREE, source_code, state, language
        )
        no_change = source_code == original_code
        no_setter_found = False
        while not no_setter_found:
            b4_code = source_code
            TREE, source_code = update_spark_conf_setters(
                TREE, source_code, state, language
            )
            no_setter_found = source_code == b4_code
    return source_code


# run(JAVA, JAVA_SOURCE_CODE)
run(SCALA, SCALA_SOURCE_CODE)
