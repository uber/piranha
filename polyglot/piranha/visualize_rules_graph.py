import argparse
import os
import re
import toml
import graphviz
from dataclasses import dataclass

# REQUIRED: pip install toml
# REQUIRED: pip install graphviz


@dataclass(frozen=True, eq=True)
class Edge:
    to: str
    scope: str


def collect_rules_groups_edges():
    for config_path in args.configurations_path:
        config_path = os.path.abspath(config_path) + os.path.sep
        rules_file = config_path + 'rules.toml'

        # rules.toml should exist, deliberately fail otherwise
        rules_toml_dict = toml.load(rules_file)
        collect_rules_and_groups(rules_toml_dict)

        # edges are optional for input rules (i.e., non built-in)
        edges_file = config_path + 'edges.toml'
        if os.path.exists(edges_file):
            edges_toml_dict = toml.load(edges_file)
            collect_edges(edges_toml_dict)


def collect_rules_and_groups(rules_toml_dict):
    """
    Collects rules and groups to further build graph nodes.

    Rules belonging to a group different than `Cleanup Rule` are displayed below the group name.

    Cleanup Rule's are added to a set as they are displayed differently.
        They get an additional label `(Cleanup_Rule)`.

    Nodes without a group are added to a different set.
    """
    for rule_toml in rules_toml_dict['rules']:
        rule_name: str = sanitize_name(rule_toml['name'])
        if 'groups' in rule_toml:
            rule_groups = rule_toml['groups']
            collect_node_for_group_rule(rule_name, rule_groups)
        else:  # rule without group
            orphan_nodes.add(rule_name)


def collect_node_for_group_rule(rule_name: str, rule_groups: 'list[str]'):
    for group_name in rule_groups:
        group_name = sanitize_name(group_name)
        if group_name != 'Cleanup_Rule':
            if group_name in rules_by_group_dict:
                rules_by_group_dict[group_name].append(rule_name)
            else:
                rules_by_group_dict[group_name] = [rule_name]
        else:  # we don't want to group 'Cleanup Rule's under the same shape
            cleanup_rules.add(rule_name)


def sanitize_name(s: str) -> str:
    """Graphviz does not like names with spaces. Converts spaces to '_'."""
    s = re.sub(r"\s+", '_', s)
    return s


def collect_edges(edges_toml_dict):
    """
    Groups outgoing edges by rule/group.

    All names are sanitized to replace empty spaces by `_`.
    """
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


def group_rule_names_label(rule_names: 'list[str]') -> 'list[str]':
    return [inline_name(rule_name) for rule_name in rule_names]


def inline_name(rule_name: str) -> str:
    """Should be called for rules under a group."""
    if rule_name in cleanup_rules:
        return f'{rule_name} (Cleanup Rule)'
    else:
        return rule_name


def initialize_graph() -> graphviz.Digraph:
    graph_attr = {
        'label': str(args.title),
        'labelloc': 't',
        'fontsize': '30'
    }
    graph = graphviz.Digraph(filename=output_file_path, graph_attr=graph_attr)
    graph.format = 'svg'
    return graph


def generate_graph_nodes():
    generate_non_orphan_nodes()
    generate_orphan_nodes()


def generate_non_orphan_nodes():
    for rule_name in all_non_orphan_rule_names():
        if rule_name in rules_by_group_dict:
            generate_node_for_group_rule(rule_name)
        else:  # a rule not in a group
            generate_node_for_non_group_rule(rule_name)

        added_nodes.add(rule_name)


def all_non_orphan_rule_names() -> 'list[str]':
    node_names_with_only_outgoing_edges = [
        node_name for node_name in rules_by_group_dict.keys() if node_name not in outgoing_edges_by_node.keys()
    ]

    non_orphan_rule_names = list(outgoing_edges_by_node.keys())
    non_orphan_rule_names.extend(node_names_with_only_outgoing_edges)
    non_orphan_rule_names.extend(cleanup_rules)
    return non_orphan_rule_names


def generate_node_for_group_rule(rule_name: str):
    rule_names_in_group = rules_by_group_dict[rule_name]
    for group_rule_name in rule_names_in_group:
        added_nodes.add(group_rule_name)

    rule_names_label: 'list[str]' = group_rule_names_label(
        rule_names_in_group)
    node_label = rule_name + '\\n\\n' + '\\n'.join(rule_names_label)

    graph.node(rule_name, node_label, shape='record')


def generate_node_for_non_group_rule(rule_name: str):
    if rule_name not in added_nodes:
        if rule_name in cleanup_rules:
            node_label = f'{rule_name}\\n(Cleanup Rule)'
            graph.node(rule_name, node_label)
        else:
            graph.node(rule_name)


def generate_orphan_nodes():
    # orphan_nodes don't have any group, not even 'Cleanup_Rule'
    for node in orphan_nodes:
        # avoid adding a node that had a label before
        if node not in added_nodes:
            added_nodes.add(node)
            graph.node(node)


def generate_graph_edges():
    for node, edges in outgoing_edges_by_node.items():
        for edge in edges:
            graph.edge(node, edge.to, edge.scope)


###########
# Arguments
example_text = '''For a complete output, the script needs the directory with the built-in rules for a given language. Example:

        python visualize_rules_graph.py ./ff_cleanup.dot src/cleanup_rules/java demo/feature_flag_cleanup/java/configurations --title "Feature Flag Cleanup"

    Experimental:
        The generated graph may end very wide if you have several rules with no outgoing edges.
        You can experiment passing the `--unflatten` option and changing the values of `--stagger` (https://graphviz.readthedocs.io/en/stable/manual.html#unflatten).
        Another option is to manually edit the generated .dot file to include invisible edges (https://stackoverflow.com/a/11136488/1008952).
    '''

description_text = '''Script to output a .dot graph and svg image for visualizing how rules/groups are connected.
Please install the `toml` PyPi package: `python install toml`.
Please install the `graphviz` PyPi package: `python install graphviz`.

The script assumes that rules will have at most [0,2] groups. If a rule has two groups, one will be `Cleanup Rule`. The visualization will likely break if any rule has > 2 groups.'''

arg_parser = argparse.ArgumentParser(
    description=description_text, epilog=example_text, formatter_class=argparse.RawDescriptionHelpFormatter)
arg_parser.add_argument('output_file_path', type=str,
                        help="Path and file name/extension for the output file.")
arg_parser.add_argument('configurations_path', type=str, nargs='+',
                        help="One or more root directory that contains 'rules.toml' and 'edges.toml'")
arg_parser.add_argument('--title', nargs='?', default='',
                        help='Optional title for the graph')
arg_parser.add_argument('--unflatten', action='store_true', default=False)
arg_parser.add_argument('--stagger', nargs='?', default=2, const=2, type=int)
args = arg_parser.parse_args()

###########
# Execution
rules_by_group_dict: 'dict[str, list[str]]' = {}
outgoing_edges_by_node: 'dict[str, list[Edge]]' = {}

cleanup_rules: 'set[str]' = set()
orphan_nodes: 'set[str]' = set()

output_file_path = os.path.abspath(args.output_file_path)

if os.path.isdir(output_file_path):
    raise ValueError(
        'output_file_path (first arg) should be a file, not a directory')

output_dir_path = os.path.dirname(output_file_path)
if not os.path.exists(output_dir_path):
    os.makedirs(output_dir_path)

collect_rules_groups_edges()

graph = initialize_graph()
added_nodes: 'set[str]' = set()
generate_graph_nodes()
generate_graph_edges()

if args.unflatten:
    graph.unflatten(stagger=args.stagger).render()
else:
    graph.render()
