from polyglot_piranha import Rule, Edge, Constraint, RuleGraph
from query_utils import java_import_declaration_type, filter_eq, java_type_identifier, filter_not_eq, java_method_invocation, java_file, query, rename_tags, transform_tags


udi_query = query(java_import_declaration_type, [
                  filter_eq("imported_type", "org.joda.time.Duration")])

cns = Constraint(matcher=query(java_file, []),
                 queries=[query(transform_tags(java_import_declaration_type, lambda i: 'c_' + i),
                                [filter_eq('c_type_qualifier', 'org.joda.time'), filter_not_eq('c_type_name', 'Duration')]),
                          query(java_type_identifier, [filter_eq("type_identifier", "Duration")])])
update_duration_import = Rule('update_duration_import', query= udi_query, replace='java.time.Duration', replace_node='imported_type',
    constraints= [cns])

# cns = Constraint(matcher="(program) @prg",
#                  queries=["""
# (
# (import_declaration (scoped_identifier (scoped_identifier)@c_fqn (identifier)@c_name) @c_type) @c_i
# (#eq? @c_fqn "org.joda.time")
# (#not-eq? @c_name "Duration")
# )
# """,
#                           """
# (type_identifier) @ty_id
# (#eq? @ty_id "Duration")
# """])


update_standard_hours = Rule("update_standard_hours",
                             query=query(java_method_invocation, [
                                         filter_eq("receiver", "@type_name"), filter_eq("name", "standardHours")]),
                             replace="ofHours",
                             replace_node="method_invocation.name",
                             holes=["type_name"])


rg = RuleGraph(["java"])



rg = RuleGraph(["java"])
rg.add_rule(update_duration_import)
rg.add_rule(update_standard_hours)

rg.add_edge(Edge("update_duration_import", ["update_standard_hours"], "File"))

rules, edges, piranha_arguments = rg.as_tomls()
output = rg.apply("/Users/ketkara/repositories/open-source/test_piranha_config",
                             "/Users/ketkara/repositories/open-source/uber_piranha/polyglot/piranha/test-resources/java/joda_to_java/only_expressions_usage/input",
                             "/Users/ketkara/repositories/open-source/test_piranha_config")

print(output)
print(rules)
print("------")
print(edges)
print("-------")
print(piranha_arguments)
