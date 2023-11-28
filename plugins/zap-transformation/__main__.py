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


# This function creates a list of simple rules for the following scenario:
# - zap.Any is used with a pattern from the mapping
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

# This function creates a rule graph for the following scenario:
# - zap.Any is used with a variable (e.g., zap.Any("key", &value))
# - The variable is declared within the enclosing function
# - The variable is initialized to a pattern from the mapping
def rule_graph_for_usage_with_variables(mapping):
    # Matches the usage of zap.Any with a variable
    usage_with_var = Rule(
        f"usage_of_zap_any_with_var",
        query=f"cs logger.With(zap.Any(:[f], &:[var_name])).Info(:[i])",
        holes = {"zap"}
    )
    rules =[usage_with_var]
    edges = []
    for api, patterns in mapping.items():
        for pattern in patterns:
            # Matches the declaration of a variable initialized to the pattern
            var_decl_pattern = Rule(
                name=f"var_decl_{api}_usage_{pattern}",
                query=f"cs :[var_name] = {pattern}",
                holes = {"var_name"},
                is_seed_rule=False
            )
            # Update the zap any usage
            update_zap_usage = Rule(
                name=f"update_zap_usage_{api}_usage_{pattern}",
                query=f"cs logger.With(zap.Any(:[f], &:[var_name])).Info(:[i])",
                replace_node="*",
                replace=f"logger.With(zap.{api}(@f, &@var_name)).Info(@i)",
                holes = {"var_name"},
                is_seed_rule=False
            )
            # When zap usage with a variable is found, we find the declaration of the variable within the enclosing function
            edges.append(OutgoingEdges(
                usage_with_var.name,
                to = [ var_decl_pattern.name ],
                scope= "Function-Method"
            ))
            # If the variable declaration is found for that variable, and it is initialized to the pattern, we update the zap usage
            edges.append(OutgoingEdges(
                var_decl_pattern.name,
                to = [ update_zap_usage.name ],
                scope= "Function-Method"
            ))
            rules.extend([var_decl_pattern, update_zap_usage])

    return rules, edges


simple_rules = get_simple_rule(mapping)
complex_rules, edges = rule_graph_for_usage_with_variables(mapping)

rule_graph = RuleGraph(rules=simple_rules + complex_rules, edges=edges)
args = PiranhaArguments(
    "go",
    paths_to_codebase=["/Users/ketkara/repositories/open-source/piranha/plugins/zap-transformation/resource"],
    substitutions={"zap": "zap"},
    rule_graph=rule_graph,
)

summary = execute_piranha(args)

print(summary)
