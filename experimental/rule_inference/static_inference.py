import attr
from tree_sitter import Node, TreeCursor
from typing import List, Dict
from patch import Patch
from node_utils import NodeUtils
import re


@attr.s
class QueryWriter:
    """
    This class writes a query for a given node considering its depth and prefix.
    """

    capture_groups = attr.ib(factory=dict)
    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(factory=list)
    outer_most_node = attr.ib(default=None)

    def write(self, seq_nodes: List[Node]):
        """
        Get textual representation of the sequence.
        Find for each named child of source_node, can we replace it with its respective target group.
        """
        node_queries = [self.write_query(node) for node in seq_nodes]

        self.query_str = ".".join(node_queries) + "\n" + "\n".join(self.query_ctrs)
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
                prefix = (
                    f"{cursor.current_field_name()}: "
                    if cursor.current_field_name()
                    else ""
                )
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

    def replace_with_tags(self, replace_str: str) -> str:
        for capture_group, node in sorted(
            self.capture_groups.items(), key=lambda x: -len(x[1].text)
        ):
            text_repr = NodeUtils.convert_to_source(node)
            if text_repr in replace_str:
                replace_str = replace_str.replace(text_repr, f"{capture_group}")
        return replace_str


@attr.s
class Inference:
    nodes_before = attr.ib(type=List[Node])
    nodes_after = attr.ib(type=List[Node])
    name = attr.ib(init=False)

    _counter = 0

    def __attrs_post_init__(self):
        type(self)._counter += 1
        self.name = f"rule_{type(self)._counter}"

    def static_infer(self) -> str:
        if len(self.nodes_after) > 0 and len(self.nodes_before) > 0:
            return self.create_replacement()
        elif len(self.nodes_after) > 0:
            # We don't support additions for now. TODO
            pass
        elif len(self.nodes_before) > 0:
            return self.create_deletion()

    def find_nodes_to_change(self, node_before: Node, node_after: Node):
        """
        Function to find nodes to change.
        """
        diverging_nodes = []
        if node_before.type == node_after.type:
            # Check if there's only one and only one diverging node
            # If there's more than one, then we can't do anything
            diverging_nodes = [
                (child_before, child_after)
                for child_before, child_after in zip(
                    node_before.named_children, node_after.named_children
                )
                if NodeUtils.convert_to_source(child_before)
                != NodeUtils.convert_to_source(child_after)
            ]

        if len(diverging_nodes) == 1:
            return self.find_nodes_to_change(*diverging_nodes[0])

        return node_before, node_after

    def create_replacement(self) -> str:
        """
        Create a rule based on the node before and after.
        """
        # For replacements (---- +++++)
        if len(self.nodes_before) == 1:
            if len(self.nodes_after) == 1:
                self.nodes_before[0], self.nodes_after[0] = self.find_nodes_to_change(
                    self.nodes_before[0], self.nodes_after[0]
                )
            qw = QueryWriter()
            query = qw.write([self.nodes_before[0]])
            lines_affected = " ".join(
                [NodeUtils.convert_to_source(node) for node in self.nodes_after]
            )
            replacement_str = qw.replace_with_tags(lines_affected)

            rule = f'''[[rules]]\n\nname = \"{self.name}\"\n\nquery = """{query}"""\n\nreplace_node = "{qw.outer_most_node}"\n\nreplace = "{replacement_str}"'''
            return rule

        else:
            # find the smallest common acesotr of _nodes_before
            ancestor = NodeUtils.find_lowest_common_ancestor(self.nodes_before)
            replacement_str = NodeUtils.convert_to_source(
                ancestor, exclude=self.nodes_before
            )

            replacement_str = replacement_str.replace(
                "{placeholder}", "", len(self.nodes_before) - 1
            )

            lines_affected = " ".join(
                [NodeUtils.convert_to_source(node) for node in self.nodes_after]
            )
            replacement_str = replacement_str.replace(
                "{placeholder}", lines_affected, 1
            )

            qw = QueryWriter()
            query = qw.write([ancestor])
            replacement_str = qw.replace_with_tags(replacement_str)

            rule = f'''[[rules]]\n\nname = \"{self.name}\"\n\nquery = """{query}"""\n\nreplace_node = "{qw.outer_most_node}"\n\nreplace = "{replacement_str}"'''
            return rule

    def create_deletion(self) -> str:
        if len(self.nodes_before) == 1:
            node_before = self.nodes_before[0]
            qw = QueryWriter()
            query = qw.write([node_before])
            replace_str = ""
            rule = f'''[[rules]]\n\nname = \"{self.name}\"\n\nquery = """{query}"""\n\nreplace_node = "{qw.outer_most_node}"\n\nreplace = "{replace_str}"'''
            return rule

        pass

    def create_addition(self) -> str:
        pass
