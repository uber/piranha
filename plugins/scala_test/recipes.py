from polyglot_piranha import (
    Rule,
    OutgoingEdges,
    RuleGraph,
    PiranhaArguments,
    execute_piranha,
)


def replace_imports(
    target_new_types: dict[str, str],
    search_heuristic: str,
    path_to_codebase: str,
    dry_run=False,
):
    """This function replaces the imports of the target types with the new types.
    The search heuristic is used to find the files that contain the target types.

    Args:
        target_new_types (dict[str, str]): A dictionary from target type to new type (fully qualified names)
        search_heuristic (str): The search heuristic to find the files that contain the target types
        path_to_codebase (str): The path to the codebase
        dry_run (bool, optional): True if the changes should not be written to disk. Defaults to False.

    Returns:
        _type_: A list of PiranhaOutput objects
    """
    find_relevant_files = Rule(
        name="find_relevant_files",
        query='((identifier) @x (#eq? @x "@search_heuristic"))',
        holes={"search_heuristic"},
    )
    find_relevant_files_andThen_update_import = OutgoingEdges(
        "find_relevant_files", to=["update_import"], scope="File"
    )

    rules = [find_relevant_files]
    edges = [find_relevant_files_andThen_update_import]

    for target_type, new_type in target_new_types.items():
        rs, es = replace_import_rules_and_edges(target_type, new_type)
        rules.extend(rs)
        edges.extend(es)

    rule_graph = RuleGraph(rules=rules, edges=edges)

    args = PiranhaArguments(
        language="scala",
        path_to_codebase=path_to_codebase,
        rule_graph=rule_graph,
        substitutions={"search_heuristic": f"{search_heuristic}"},
        dry_run=dry_run,
    )

    return execute_piranha(args)


def replace_import_rules_and_edges(
    target_qualified_type_name: str, new_qualified_type_name: str
) -> (list[Rule], list[OutgoingEdges]):
    """This function generates the rules and edges to replace the imports of the target type with the new type.
    It supports both simple and nested imports. While the simple imports are replaced directly, the nested imports are deleted and the new type is imported (as a simple non-nested import).
    Assume that the target type is "a.b.c.d" and the new type is "x.y.z". Then the following rules are generated:
    import a.b.c.d -> import x.y.z
    import a.b.c.{d, e} -> import x.y.z \n import a.b.c.{d}
    """
    name_components = target_qualified_type_name.split(".")
    type_name = name_components[-1]

    qualifier_predicate = "\n".join(
        [f'(#match? @import_decl "{n}")' for n in name_components[:-1]]
    )

    delete_nested_import = Rule(
        name=f"delete_nested_import_{type_name}",
        query=f"""(
            (import_declaration (namespace_selectors (_) @tn )) @import_decl
            (#eq? @tn "{type_name}")
            {qualifier_predicate}
        )""",
        replace_node="tn",
        replace="",
        is_seed_rule=False,
        groups={"update_import"},
    )

    update_simple_import = Rule(
        name=f"update_simple_import_{type_name}",
        query=f"cs import {target_qualified_type_name}",
        replace_node="*",
        replace=f"import {new_qualified_type_name}",
        is_seed_rule=False,
        groups={"update_import"},
    )

    insert_import = Rule(
        name=f"insert_import_{type_name}",
        query="(import_declaration) @import_decl",
        replace_node="import_decl",
        replace=f"@import_decl\nimport {new_qualified_type_name}\n",
        is_seed_rule=False,
    )

    e2 = OutgoingEdges(
        f"delete_nested_import_{type_name}",
        to=[f"insert_import_{type_name}"],
        scope="Parent",
    )

    return [delete_nested_import, update_simple_import, insert_import], [e2]
