import argparse
import os
import re
import toml
from dataclasses import dataclass

# REQUIRED: pip install toml

# the graph can be very wide if there are several disconnected rules
# Here are a few options:
# https://stackoverflow.com/questions/11134946/distribute-nodes-on-the-same-rank-of-a-wide-graph-to-different-lines
# unflatten is probably the best option but requires a graphviz installation
# as suggested by the SO answer, one can add invisible edges (in the generated .dot) between disconnected nodes
# Example:
# python rules_dot_graph.py src/cleanup_rules/kt test-resources/kt/feature_flag_system_1/control/configurations  --title="Kotlin Test Feature Flag System 1" > temp.dot
# Then in the .dot file:
# edge[style=invis];
# delete_all_statements_after_return -> delete_putToggleEnabled
# delete_putToggleEnabled -> delete_putToggleDisabled
# delete_putToggleDisabled -> delete_includeEvent
# edge[style=solid];
# // all the generated edges


@dataclass(frozen=True, eq=True)
class Edge:
    to: str
    scope: str


example_text = '''For a complete output, the script needs the directory with the built-in rules for a given language. Example:

    python visualize_rules_graph.py ./ff_cleanup.dot src/cleanup_rules/java demo/feature_flag_cleanup/java/configurations --title "Feature Flag Cleanup"'''

description_text = '''Script to output a .dot graph for visualizing how rules/groups are connected.
Please install the `toml` PyPi package: `python install toml`.
The script assumes that rules will have at [0,2] groups. If a rule has two, one will be `Cleanup Rule`. The visualization will likely break if any rule has > 2 groups.'''

arg_parser = argparse.ArgumentParser(
    description=description_text, epilog=example_text, formatter_class=argparse.RawDescriptionHelpFormatter)
arg_parser.add_argument('output_file_path', type=str, help="Path and file name/extension for the output file.")
arg_parser.add_argument('configurations_path', type=str, nargs='+',
                        help="One or more root directory that contains 'rules.toml' and 'edges.toml'")
arg_parser.add_argument('--title', nargs='?',
                        help='Optional title for the graph')
args = arg_parser.parse_args()


def sanitize_name(s: str) -> str:
    """Graphviz does not like names with spaces. Converts spaces to '_'."""
    s = re.sub(r"\s+", '_', s)
    return s

output_file_path = os.path.abspath(args.output_file_path)

if os.path.isdir(output_file_path):
    raise ValueError('output_file_path (first arg) should be a file, not a directory')

if not os.path.exists(os.path.dirname(output_file_path)):
    os.makedirs(os.path.dirname(output_file_path))

# dict's are ordered
rules_by_group_dict: 'dict[str, list[str]]' = {}
outgoing_edges_by_node: 'dict[str, list[Edge]]' = {}

cleanup_rules: 'set[str]' = set()
orphan_nodes: 'set[str]' = set()

for config_path in args.configurations_path:
    config_path = os.path.abspath(config_path) + os.path.sep
    rules_file = config_path + 'rules.toml'

    # rules.toml should exist, deliberately fail otherwise
    rules_toml_dict = toml.load(rules_file)

    for rule_toml in rules_toml_dict['rules']:
        rule_name: str = sanitize_name(rule_toml['name'])
        if 'groups' in rule_toml:
            for group_name in rule_toml['groups']:
                group_name = sanitize_name(group_name)
                if group_name != 'Cleanup_Rule':
                    if group_name in rules_by_group_dict:
                        rules_by_group_dict[group_name].append(rule_name)
                    else:
                        rules_by_group_dict[group_name] = [rule_name]
                else:  # we don't want to group 'Cleanup Rule's under the same shape
                    cleanup_rules.add(rule_name)
        else:  # rule without group
            orphan_nodes.add(rule_name)

    # edges are optional for input rules (i.e., non built-in)
    edges_file = config_path + 'edges.toml'
    if os.path.exists(edges_file):
        edges_toml_dict = toml.load(edges_file)

        for edge_toml in edges_toml_dict['edges']:
            from_node = sanitize_name(edge_toml['from'])
            to_nodes: 'list[str]' = edge_toml['to']
            scope = sanitize_name(edge_toml['scope'])
            for to_node in to_nodes:
                to_node = sanitize_name(to_node)
                edge = Edge(to=to_node, scope=scope)
                if from_node in outgoing_edges_by_node:
                    outgoing_edges_by_node[from_node].append(edge)
                else:
                    outgoing_edges_by_node[from_node] = [edge]


dot_lines: 'list[str]' = []
dot_lines.append('digraph {\n')

if args.title:
    dot_lines.append(f'  label    = "{args.title}"')
    dot_lines.append('  labelloc = t')
    dot_lines.append('  fontsize = 30\n')

added_nodes: 'set[str]' = set()


def group_rule_names_label(rule_names: 'list[str]') -> 'list[str]':
    return [inline_name(rule_name) for rule_name in rule_names]


def inline_name(rule_name: str) -> str:
    """Should be called for rules under a group."""
    if rule_name in cleanup_rules:
        return f'{rule_name} (Cleanup Rule)'
    else:
        return rule_name


def rule_dot_node(rule_name: str) -> str:
    """Should be called for rules as a standalone node."""
    if rule_name in cleanup_rules:
        return f'{rule_name} [label="{rule_name}\\n(Cleanup Rule)"]'
    else:
        return rule_name


node_names_with_only_outgoing_edges = [
    node_name for node_name in rules_by_group_dict.keys() if node_name not in outgoing_edges_by_node.keys()]

non_orphan_rule_names = list(outgoing_edges_by_node.keys())
non_orphan_rule_names.extend(node_names_with_only_outgoing_edges)
non_orphan_rule_names.extend(cleanup_rules)

# generating nodes
for rule_name in non_orphan_rule_names:
    if rule_name in rules_by_group_dict:
        rule_names_in_group = rules_by_group_dict[rule_name]
        for group_rule_name in rule_names_in_group:
            added_nodes.add(group_rule_name)

        rule_names_label: 'list[str]' = group_rule_names_label(
            rule_names_in_group)
        node_label = '[shape=record label="' + rule_name + \
            '\\n\\n' + '\\n'.join(rule_names_label) + '"]'
        dot_lines.append(f'  {rule_name} {node_label}')

    else:  # a rule not in a group
        if rule_name not in added_nodes:
            dot_lines.append('  ' + rule_dot_node(rule_name))

    added_nodes.add(rule_name)

# orphan_nodes don't have any group, not even 'Cleanup_Rule'
for node in orphan_nodes:
    # avoid adding a node that had a label before
    if node not in added_nodes:
        dot_lines.append('  ' + node)
        added_nodes.add(node)

dot_lines.append('\n')

# generating edges
for node, edges in outgoing_edges_by_node.items():
    for edge in edges:
        dot_lines.append('  ' + f'{node} -> {edge.to} [label="{edge.scope}"]')

dot_lines.append('\n}')

dot_source = '\n'.join(dot_lines)


with open(output_file_path, 'w') as f:
    f.write(dot_source)
