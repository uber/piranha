from polyglot_piranha import Rule, RuleGraph, execute_piranha, PiranhaArguments

# Define a rule with empty substitutions and specific query
rule = Rule(
    name="SampleRule",
    query="cs :[x].isEnabled(:[args])",
    # Include additional initialization parameters if needed
)

# Create a rule graph with the defined rule
rule_graph = RuleGraph(
    rules=[rule],
    edges=[],  # Assuming no edges for a single rule; adjust if necessary
    # Include additional initialization parameters if needed
)

# Set up the PiranhaArguments instance
piranha_args = PiranhaArguments(
    language="java",  # Adjust to the language you're working with
    rule_graph=rule_graph,
    path_to_codebase='./repos/Fabrication'
)

# Call the execute_piranha function with the PiranhaArguments instance
output = execute_piranha(piranha_args)
all_flags = set()

for summary in output:
    for mt in summary.matches:
        _, actual_match = mt
        all_flags.add(actual_match.matches['args'])

print(list(all_flags)[:10])

# Handle the output (e.g., log, print, write to file)
# ... your code to handle output ...

