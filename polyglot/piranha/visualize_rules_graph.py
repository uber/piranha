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
        if 'query' not in rule_toml:
            dummy_nodes.add(rule_name)

        if 'groups' in rule_toml:
            rule_groups = rule_toml['groups']
            collect_node_for_rule_with_group(rule_name, rule_groups)
        else:
            nodes_without_groups.add(rule_name)


def collect_node_for_rule_with_group(rule_name: str, rule_groups: 'list[str]'):
    for group_name in rule_groups:
        group_name = sanitize_name(group_name)
        if group_name != 'Cleanup_Rule':
            if group_name in rules_by_group_dict:
                rules_by_group_dict[group_name].append(rule_name)
            else:
                rules_by_group_dict[group_name] = [rule_name]
        else:  # we don't want to group `Cleanup Rule`s under the same graphviz shape
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
    generate_nodes_with_groups_and_outgoing_edges()
    # rules *without* outgoing edges and no groups have not been added yet
    # this is because we focus on outgoing edges when traversing `edges.toml`
    generate_nodes_without_groups_and_no_outgoing_edges()


def generate_nodes_with_groups_and_outgoing_edges():
    for rule_name in all_rule_names_with_groups_and_outgoing_edges():
        if rule_name in rules_by_group_dict:
            # several (n >= 1) rules under the same graphviz `record` shape.
            generate_node_for_group(rule_name)
        else:
            # not a graphviz `record` node. single rule in the shape.
            generate_node_for_rule_not_under_a_group(rule_name)

        added_nodes.add(rule_name)


def all_rule_names_with_groups_and_outgoing_edges() -> 'list[str]':
    # set difference
    # map.keys() is a set view, it doesn't have set's methods but supports operators
    node_names_with_only_outgoing_edges = rules_by_group_dict.keys() - \
        outgoing_edges_by_node.keys()

    rule_names_with_groups = list(outgoing_edges_by_node.keys())
    rule_names_with_groups.extend(node_names_with_only_outgoing_edges)
    rule_names_with_groups.extend(cleanup_rules)
    return rule_names_with_groups


def generate_node_for_group(rule_name: str):
    rule_names_in_group = rules_by_group_dict[rule_name]
    for group_rule_name in rule_names_in_group:
        added_nodes.add(group_rule_name)

    rule_names_label: 'list[str]' = [
        append_cleanup_rule_if_needed(rule_name) for rule_name in rule_names_in_group
    ]

    #############################
    # boolean_expression_simplify
    #
    # simplify_not_false
    # simplify_not_true
    # ...
    #############################
    node_label = rule_name + '\\n\\n' + '\\n'.join(rule_names_label)

    graph.node(rule_name, node_label, shape='record')


def append_cleanup_rule_if_needed(rule_name: str) -> str:
    """
    Should be called for rules under a group.

    If a rule is a cleanup rule, we append the label to the node's name *on the same line*.
    Cleanup Rules are treated differently because:
      1) Currently, there are no edges to `Cleanup Rule` (as opposed to other groups)
      2) nodes may have another group with incoming/outgoing edges.

    We want to display rules under a group which indicates flow (i.e., has edges).
    At the same time, we still want to indicate in the graph that a rule is a cleanup rule.
    """
    if rule_name in cleanup_rules:
        return f'{rule_name} (Cleanup Rule)'
    else:
        return rule_name


def generate_node_for_rule_not_under_a_group(rule_name: str):
    """The rule will be a standalone node; we can add (Cleanup Rule) *on a new line* if needed."""
    if rule_name not in added_nodes:
        if rule_name in dummy_nodes:
            graph.node(rule_name, shape='doubleoctagon')
        elif rule_name in cleanup_rules:
            node_label = f'{rule_name}\\n(Cleanup Rule)'
            graph.node(rule_name, node_label)
        else:
            graph.node(rule_name)


def generate_nodes_without_groups_and_no_outgoing_edges():
    # nodes that don't have any group, not even 'Cleanup_Rule'
    # and also no outgoing edge
    # i.e., leaf nodes with no groups -> standalone shape
    for node in nodes_without_groups:
        # avoid adding a node already added through an edge
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
nodes_without_groups: 'set[str]' = set()
# nodes without `query`
dummy_nodes: 'set[str]' = set()

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
