from polyglot_piranha import Rule, Edge, Constraint, RuleGraph
from query_utils import java_import_declaration_type, filter_eq, java_method_invocation, java_method_declaration, java_file


udi_query = query(java_import_declaration_type, [filter_eq("imported_type", "org.joda.time.Duration")])

cns = Constraint(matcher=query(java_file, []),
                 queries=[ query
                    
                    """
(
(import_declaration (scoped_identifier (scoped_identifier)@c_fqn (identifier)@c_name) @c_type) @c_i
(#eq? @c_fqn "org.joda.time")
(#not-eq? @c_name "Duration")
)
""",
                          """
(type_identifier) @ty_id
(#eq? @ty_id "Duration")
"""])


cns = Constraint(matcher="(program) @prg",
                 queries=["""
(
(import_declaration (scoped_identifier (scoped_identifier)@c_fqn (identifier)@c_name) @c_type) @c_i
(#eq? @c_fqn "org.joda.time")
(#not-eq? @c_name "Duration")
)
""",
                          """
(type_identifier) @ty_id
(#eq? @ty_id "Duration")
"""])

update_standard_hours = Rule("update_standard_hours",
                            query=query(java_method_invocation, [eq("receiver", "@type_name"), eq("name", "standardHours")]),
                            replace="ofHours",
                            replace_node="method_invocation.name",
                            holes=["duration.type"])


rg = RuleGraph(["java"])

for (kind, file_name, line_no) in data: # ensure that things are ordered by file by line number descending 
    if kind == 'function':
        r =Rule ("r", query = java_method_declaration, replace="", replace_node = "method_declaration", line= line_no, file_name = file_name)
    if kind == 'var':
        r = Rule ("r", query = java_var, replace="", replace_node = "var",  line= line_no, file_name = file_name)
    rg.add_rule(r)


rg.apply(path_to_piranha_bin, path_to_target, path_to_configurations)




rg = RuleGraph(["java"])
rg.add_rule(update_duration_import)
rg.add_rule(update_standard_hours)

rg.add_edge(Edge("update_duration_import", ["update_standard_hours"], "File"))

rules, edges, piranha_arguments = rg.as_tomls()
matches. rewrites = rg.apply("/Users/ketkara/repositories/open-source/test_piranha_config",
         "/Users/ketkara/repositories/open-source/uber_piranha/polyglot/piranha/test-resources/java/joda_to_java/only_expressions_usage/input",
         "/Users/ketkara/repositories/open-source/test_piranha_config")


print(rules)
print("------")
print(edges)
print("-------")
print(piranha_arguments)
