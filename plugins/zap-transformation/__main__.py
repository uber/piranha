import re
from polyglot_piranha import OutgoingEdges, Rule, RuleGraph, execute_piranha, PiranhaArguments


mapping = {
    "Bool": ["true", "false"],
    "Complex128": ["complex128(:[x])"],
    "Float64": ["float64(:[x])"],
}


def convert_hole_style(s):
    pattern = re.compile(r':\[(\w+)\]')
    return pattern.sub(r'@\1', s)



def get_simple_rule(mapping):
    return [
        Rule(
            f"simple_{api}_usage_{pattern}", # Create a unique name for the rule
            query=f"cs logger.With(zap.Any(:[f], {pattern})).Info(:[i])", # The query to match
            replace_node="*",
            replace="logger.With(zap.{}(@f, {})).Info(@i)".format(api, convert_hole_style(pattern)), # The replacement (the replacement is a bit awkward because we need to convert the hole style)
            holes = {"zap"}
        )
        for api, patterns in mapping.items()
        for pattern in patterns
    ]


def get_complex_rule_graph(mapping):

    # Make a rule that matches the usage of zap.Any with a variable
    usage_with_var = Rule(
        f"usage_of_zap_any_with_var",
        query=f"cs logger.With(zap.Any(:[f], &:[var_name])).Info(:[i])",
        holes = {"zap"}
    )
    rules =[usage_with_var]
    edges = []
    for api, patterns in mapping.items():
        for pattern in patterns:
            var_decl_pattern = Rule(
                name=f"var_decl_{api}_usage_{pattern}",
                query=f"cs :[var_name] = {pattern}",
                holes = {"var_name"},
                is_seed_rule=False
            )
            update_zap_usage = Rule(
                name=f"update_zap_usage_{api}_usage_{pattern}",
                query=f"cs logger.With(zap.Any(:[f], &:[var_name])).Info(:[i])",
                replace_node="*",
                replace=f"logger.With(zap.{api}(@f, &@var_name)).Info(@i)",
                holes = {"var_name"},
                is_seed_rule=False
            )
            edge1 = OutgoingEdges(
                var_decl_pattern.name,
                to = [ update_zap_usage.name ],
                scope= "Function-Method"
            )

            edge2 = OutgoingEdges(
                usage_with_var.name,
                to = [ var_decl_pattern.name ],
                scope= "Function-Method"
            )
            rules.extend([var_decl_pattern, update_zap_usage])
            edges.extend([edge1, edge2])

    return rules, edges


simple_rules = get_simple_rule(mapping)
complex_rules, edges = get_complex_rule_graph(mapping)

rule_graph = RuleGraph(rules=simple_rules + complex_rules, edges=edges)
args = PiranhaArguments(
    "go",
    paths_to_codebase=["/Users/ketkara/repositories/open-source/piranha/plugins/zap-transformation/resource"],
    substitutions={"zap": "zap"},
    rule_graph=rule_graph,
)

summary = execute_piranha(args)

print(summary)
