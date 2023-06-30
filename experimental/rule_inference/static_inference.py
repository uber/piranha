from typing import List

import attr
from comby import Comby
from experimental.rule_inference.utils.node_utils import NodeUtils
from experimental.rule_inference.utils.rule_utils import RawRule
from tree_sitter import Node, TreeCursor


@attr.s
class QueryWriter:
    """
    This class writes a query for a given node considering its depth and prefix.
    """

    seq_nodes = attr.ib(type=list)
    capture_groups = attr.ib(factory=dict)
    count = attr.ib(default=0)
    query_str = attr.ib(default="")
    query_ctrs = attr.ib(factory=list)
    outer_most_node = attr.ib(default=None)

    def write(self, simplify=False):
        """
        Get textual representation of the sequence.
        Find for each named child of source_node, can we replace it with its respective target group.
        """

        if self.query_str:
            return self.query_str

        node_queries = [
            self.write_query(node, simplify=simplify) for node in self.seq_nodes
        ]

        self.query_str = ".".join(node_queries) + "\n" + "\n".join(self.query_ctrs)
        self.query_str = f"({self.query_str})"

        return self.query_str

    def write_query(self, node: Node, depth=0, prefix="", simplify=False):
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
                if simplify:
                    s_exp += (
                        " " * (depth + 1) + f"({child_node.type}) @tag{self.count}n"
                    )
                    self.count += 1
                    if child_node.child_count == 0:
                        self.query_ctrs.append(
                            f"(#eq? @tag{self.count}n \"{child_node.text.decode('utf8')}\")"
                        )
                else:
                    s_exp += self.write_query(cursor.node, depth + 1, prefix, simplify)
            next_child = cursor.goto_next_sibling()

        self.count += 1
        node_name = f"@tag{self.count}n"
        self.capture_groups[node_name] = node

        # if the node is an identifier, add it to eq constraints
        if node.child_count == 0:
            self.query_ctrs.append(f"(#eq? {node_name} \"{node.text.decode('utf8')}\")")

        self.outer_most_node = node_name
        return s_exp + f") {node_name}"

    def simplify_query(self, capture_group):
        """Simplify a query removing all the children of capture_group and replacing it with a wildcard node
        This should be replaced with piranha at some point"""

        comby = Comby()
        match = f"(:[[node_name]] :[_]) {capture_group}"
        rewrite = f"(:[[node_name]]) {capture_group}"
        self.query_str = comby.rewrite(self.query_str, match, rewrite)

        # Now for every child of capture_group, we need to remove equality checks from the query
        stack = [self.capture_groups[capture_group]]
        while stack:
            first = stack.pop()
            to_remove = next(
                key for key, value in self.capture_groups.items() if value == first
            )
            match = f"(#eq? {to_remove} :[_])"
            self.query_str = comby.rewrite(self.query_str, match, "")
            self.capture_groups.pop(to_remove, None)
            for child in first.named_children:
                stack.append(child)

    def replace_with_tags(self, replace_str: str) -> str:
        """This logic is wrong"""
        for capture_group, node in sorted(
            self.capture_groups.items(), key=lambda x: -len(x[1].text)
        ):
            if capture_group not in self.capture_groups.keys():
                continue
            text_repr = NodeUtils.convert_to_source(node)
            if text_repr in replace_str:
                # self.simplify_query(capture_group)
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

    def static_infer(self) -> RawRule:
        if len(self.nodes_after) > 0 and len(self.nodes_before) > 0:
            return self.create_replacement()
        elif len(self.nodes_after) > 0:
            raise self.create_addition()
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

        if (
            len(diverging_nodes) == 1
            and node_before.child_count == node_after.child_count
        ):
            return self.find_nodes_to_change(*diverging_nodes[0])

        return node_before, node_after

    def create_replacement(self) -> RawRule:
        """
        Create a rule based on the node before and after.
        """
        # For replacements (---- +++++)
        if len(self.nodes_before) == 1:
            if len(self.nodes_after) == 1:
                self.nodes_before[0], self.nodes_after[0] = self.find_nodes_to_change(
                    self.nodes_before[0], self.nodes_after[0]
                )
            qw = QueryWriter([self.nodes_before[0]])
            qw.write()
            lines_affected = " ".join(
                [NodeUtils.convert_to_source(node) for node in self.nodes_after]
            )
            replacement_str = qw.replace_with_tags(lines_affected)

            return RawRule(
                name=self.name,
                query=qw.query_str,
                replace_node=qw.outer_most_node[1:],
                replace=replacement_str,
            )

        else:
            # find the smallest common ancestor of _nodes_before
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

            qw = QueryWriter([ancestor])
            qw.write()
            replacement_str = qw.replace_with_tags(replacement_str)

            return RawRule(
                name=self.name,
                query=qw.query_str,
                replace_node=qw.outer_most_node[1:],
                replace=replacement_str,
            )

    def create_deletion(self) -> RawRule:
        if len(self.nodes_before) == 1:
            node_before = self.nodes_before[0]
            qw = QueryWriter([node_before])
            query = qw.write()
            return RawRule(
                name=self.name,
                query=query,
                replace_node=qw.outer_most_node[1:],
                replace="",
            )

        raise NotImplementedError

    def create_addition(self) -> str:
        raise NotImplementedError
