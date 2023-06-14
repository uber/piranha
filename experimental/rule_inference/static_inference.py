from tree_sitter import Node, TreeCursor
from typing import List, Dict
from patch import Patch
from node_utils import NodeUtils
import attr


@attr.s
class QueryWriter:
    capture_groups = attr.ib(factory=dict)
    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(factory=list)
    outer_most_node = attr.ib(default=None)

    def write(self, seq_nodes: List[Node]):
        """
        Get textual representation of the sequence.
        Then find for each named child of source_node, can we replace it with its respective target group.
        """
        node_queries = []
        for node in seq_nodes:
            node_queries.append(self.write_query(node))

        self.query_str = ".".join(node_queries)
        self.query_str += "\n"
        self.query_str += "\n".join(self.query_ctrs)
        self.query_str = f"({self.query_str})"

        return self.query_str

    def write_query(self, node: Node, depth=0, prefix=""):
        """
        Write a query for a given node, considering its depth and prefix.
        """
        indent = " " * depth
        cursor: TreeCursor = node.walk()
        s_exp = indent + f"{prefix}({node.type} "

        next_child = cursor.goto_first_child()
        while next_child:
            child_node: Node = cursor.node
            if child_node.is_named:
                s_exp += "\n"
                prefix = ""
                if cursor.current_field_name():
                    prefix = f"{cursor.current_field_name()}: "
                s_exp += self.write_query(cursor.node, depth + 1, prefix)
            next_child = cursor.goto_next_sibling()

        self.count += 1
        self.capture_groups[f"@tag{self.count}"] = node

        # if the node is an identifier, add it to eq constraints
        if node.child_count == 0:
            self.query_ctrs.append(
                f"(#eq? @tag{self.count} \"{node.text.decode('utf8')}\")"
            )

        self.outer_most_node = f"@tag{self.count}"
        return s_exp + f") @tag{self.count}"


# Basic mapping. Matches nodes orderly.
def find_mappings(
    nodes_before: List[Node], nodes_after: List[Node]
) -> Dict[int, List[Node]]:
    nodes_before = NodeUtils.get_smallest_nonoverlapping_set(nodes_before)
    nodes_after = NodeUtils.get_smallest_nonoverlapping_set(nodes_after)

    # compute mapping by search for isomorphic nodes

    mapping = {}
    for before, after in zip(nodes_before, nodes_after):
        mapping[before.id] = [after]

    if len(nodes_before) > len(nodes_after):
        for node_before in nodes_before[len(nodes_after) :]:
            mapping[node_before.id] = []

    if len(nodes_before) < len(nodes_after):
        for node_after in nodes_after[len(nodes_before) :]:
            mapping[nodes_before[-1].id].append(node_after)

    return mapping


def find_nodes_to_change(node_before: Node, node_after: Node):
    diverging_nodes = []
    if node_before.type == node_after.type:
        # Check if there's only one and only one diverging node
        # If there's more than one, then we can't do anything
        for child_before, child_after in zip(node_before.children, node_after.children):
            if NodeUtils.convert_to_source(child_before) != NodeUtils.convert_to_source(
                child_after
            ):
                diverging_nodes.append((child_before, child_after))

    if len(diverging_nodes) == 1:
        return find_nodes_to_change(*diverging_nodes[0])

    return node_before, [node_after]


# given a mapping , create rule always works
# all targets connect to a source
# the idea is then replace the source with targets if they are different,
# then you will always end up with the correct result, right?
def create_rule(node_before: Node, node_afters: List[Node]) -> str:
    """
    1. If it's an insertion.
        i. create a rule that inserts with a not contains filter to prevent recursive application
    2. If it's a deletion.
        i. simply create the match for the node with all eqs and delete the outer_most
    3. if it's a replacement:
        i. create a matcher for the nodes to be replaced
        ii. get a textual representation of the target nodes (unparse)
        iii. for each capture group in the matcher, check if it's possible to replace any substring of the target with the capture group
        iv. make sure the outermost left node is not the in replacement string, otherwise ifninite recursion. prevent this with a not_contains filter
    """

    # For replacements (---- +++++)

    if len(node_afters) == 1:
        # Find whether we need to replace the entire node or just a part of it
        node_before, node_afters = find_nodes_to_change(node_before, node_afters[0])

    qw = QueryWriter()
    query = qw.write([node_before])

    replace_str = "\\n".join(
        [NodeUtils.convert_to_source(node_after) for node_after in node_afters]
    )

    # Prioritize the longest strings first
    for capture_group, node in sorted(
        qw.capture_groups.items(), key=lambda x: -len(x[1].text)
    ):
        text_repr = NodeUtils.convert_to_source(node)
        if text_repr in replace_str:
            replace_str = replace_str.replace(text_repr, f"{capture_group}")

    rule = f'''query = """{query}"""\n\nreplace_node = "{qw.outer_most_node}"\n\nreplace = "{replace_str}"'''

    # Check if the outermost node is in the replacement string
    # If so then we need to add a not_contains filter to prevent infinite recursion
    if qw.outer_most_node in replace_str:
        # Idea for a filter. The parent of node_before should not contain the outer_most_node
        # This is not a perfect filter, but it should work for most cases
        enclosing_node = f"({node_afters[0].type}) @parent"
        qw = QueryWriter(count=qw.count + 1)
        query = qw.write([node_afters[0]])
        not_contains = f"{query}"
        rule += f'''\n\nenclosing_node = "{enclosing_node}"\n\nnot_contains = """[{not_contains}]"""'''

    return rule
