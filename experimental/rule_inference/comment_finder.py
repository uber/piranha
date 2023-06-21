"""
This module provides a CommentFinder class for finding comments in an abstract syntax tree (AST) and their associated
replacement pairs, as well as removing partial nodes from the pairs.
"""

from collections import defaultdict, deque
from typing import List, Dict, Deque, Tuple

import attr
from tree_sitter import Tree, Node

from experimental.rule_inference.node_utils import NodeUtils


@attr.s
class CommentFinder:
    """
    CommentFinder traverses an AST to find nodes of type comment. Each comment will be associated with a number, and
    it tells us when we should start collecting nodes for the replacement pair. When we find // 1 end, we stop collecting.
    """

    source_tree = attr.ib(type=Tree)
    target_tree = attr.ib(type=Tree)
    replacement_source = attr.ib(
        type=Dict[str, List[Tree]], default=attr.Factory(lambda: defaultdict(list))
    )
    replacement_target = attr.ib(
        type=Dict[str, List[Tree]], default=attr.Factory(lambda: defaultdict(list))
    )
    edges = attr.ib(
        type=Dict[str, List[str]], default=attr.Factory(lambda: defaultdict(list))
    )

    def find_replacement_pairs(self) -> Dict[str, Tuple[List[Node], List[Node]]]:
        """
        Invokes find_replacement_pair on each tree. Finds matching pairs using the comments, returns those pairs.
        """

        source_dict = self.find_replacement_pair(self.source_tree)
        target_dict = self.find_replacement_pair(self.target_tree)

        # Collect matching pairs using the comments
        matching_pairs = {}

        for comment in source_dict:
            if comment in target_dict:
                matching_pairs[comment] = (source_dict[comment], target_dict[comment])

        return matching_pairs

    def find_replacement_pair(self, tree: Tree):
        """
        Traverses the AST to find nodes of type comment. Each comment will be associated with a number, and
        it tells us when we should start collecting nodes for the replacement pair. When we find // 1 end, we stop collecting.
        """

        root = tree.root_node
        stack: Deque = deque([root])
        comment = None
        replacement_dict = defaultdict(list)
        while stack:
            node = stack.pop()

            if node.type == "line_comment":
                prev_comment = comment
                comment = node.text.decode("utf8")
                if "->" in comment:
                    x, y = comment.split("->")
                    self.edges[x[2:].strip()].append(y.strip())
                    comment = prev_comment
                elif "end" in comment:
                    comment = None

            elif comment:
                replacement_dict[comment].append(node)
            for child in reversed(node.children):
                stack.append(child)

        for comment, nodes in replacement_dict.items():
            nodes = NodeUtils.remove_partial_nodes(nodes)
            nodes = NodeUtils.get_smallest_nonoverlapping_set(nodes)
            replacement_dict[comment] = nodes

        return replacement_dict
