from collections import defaultdict, deque
from typing import Deque, Dict, List, Set, Tuple

import attr
from tree_sitter import Node, Tree

from experimental.rule_inference.utils.node_utils import NodeUtils


@attr.s
class CommentFinder:
    """
    The CommentFinder class traverses an AST to find nodes of type comment.

    Each comment node is associated with a number indicating when the system should start
    collecting nodes for the replacement pair. When "// 1 end" is encountered, the system stops collecting.

    :param source_tree: The source tree for traversing.
    :type source_tree: Tree

    :param target_tree: The target tree for traversing.
    :type target_tree: Tree
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
        type=Dict[str, Set[str]], default=attr.Factory(lambda: defaultdict(set))
    )

    def process_trees(self) -> Dict[str, Tuple[List[Node], List[Node]]]:
        """
        This method invokes _traverse_tree on both trees, finds matching pairs using the comments,
        and updates the edges. It returns the matching pairs.

        :return: A dictionary of matching pairs.
        :rtype: Dict[str, Tuple[List[Node], List[Node]]]
        """

        source_dict = self._traverse_tree(self.source_tree)
        target_dict = self._traverse_tree(self.target_tree)

        matching_pairs = {}

        for comment in source_dict:
            if comment in target_dict:
                matching_pairs[comment] = (source_dict[comment], target_dict[comment])

        return matching_pairs

    def _traverse_tree(self, tree: Tree):
        """
        This private method contains the common traversal logic for finding comment nodes in an AST and updating edges.

        :param tree: The tree to traverse.
        :type tree: Tree
        :return: A dictionary of comment nodes and associated nodes.
        :rtype: dict
        """
        root = tree.root_node
        stack: Deque = deque([root])
        comment = None
        replacement_dict = defaultdict(list)
        while stack:
            node = stack.pop()

            if "comment" in node.type:
                prev_comment = comment
                comment = node.text.decode("utf8")
                if "->" in comment:
                    x, y = comment.split("->")
                    self.edges[x[2:].strip()].add(y.strip())
                    comment = prev_comment
                elif "end" in comment:
                    comment = None
                else:
                    comment = comment[2:].strip()

            elif comment:
                replacement_dict[comment].append(node)
            for child in reversed(node.children):
                stack.append(child)

        for comment, nodes in replacement_dict.items():
            nodes = NodeUtils.remove_partial_nodes(nodes)
            nodes = NodeUtils.get_smallest_nonoverlapping_set(nodes)
            replacement_dict[comment] = nodes

        return replacement_dict
