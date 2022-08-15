from polyglot_piranha import Rule, Edge, Constraint, RuleGraph

cns = Constraint(matcher="(program) @prg", 
queries = ["""
(
(import_declaration (scoped_identifier (scoped_identifier)@c_fqn (identifier)@c_name) @c_type) @c_i
(#eq? @c_fqn "org.joda.time")
(#not-eq? @c_name "Duration")
)
""", 
"""
(type_identifier) @ty_id
(#eq? @ty_id "Duration")
"""
]
)
update_duration_import = Rule("update_duration_import",
 query = """(
(import_declaration 
	(scoped_identifier 
    scope: (_) @duration.scop
    name: (_) @duration.type
)@duration.fqn)
(#eq? @duration.fqn "org.joda.time.Duration"))""", replace = "java.time.Duration", replace_node = "duration.fqn", 
constraints=[cns])

update_standard_hours = Rule("update_standard_hours",
 query = """(
(method_invocation 
	object: (_) @method_invocation.object
    name: (_) @method_invocation.name
    arguments : (_) @args
) @mi
(#eq? @method_invocation.object "@duration.type")
(#eq? @method_invocation.name "standardHours"))""", replace = "ofHours", replace_node = "method_invocation.name", holes = ["duration.type"])


rg = RuleGraph(["java"])
rg.add_rule(update_duration_import)
rg.add_rule(update_standard_hours)

rg.add_edge(Edge("update_duration_import", ["update_standard_hours"], "File"))

rules, edges, piranha_arguments = rg.as_tomls()
rg.apply("/Users/ketkara/repositories/open-source/test_piranha_config",
 "/Users/ketkara/repositories/open-source/uber_piranha/polyglot/piranha/test-resources/java/joda_to_java/only_expressions_usage/input",
 "/Users/ketkara/repositories/open-source/test_piranha_config" )


print(rules)
print("------")
print(edges)
print("-------")
print(piranha_arguments)