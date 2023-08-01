# Copyright (c) 2023 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

from collections import defaultdict, deque
from typing import Deque, Dict, List, Set, Tuple

import attr
from tree_sitter import Node, Tree

from piranha_playground.rule_inference.utils.node_utils import NodeUtils


@attr.s
class GraphParser:
    """
    The TemplateParser class performs depth-first search on two given Abstract Syntax Trees (ASTs) to identify
    mapping from 'before' code snippet (sub-AST) to 'after' code snippet (sub-AST) required to construct the
    rules of the graph. It also parses special comments that specify the flow between the rules.

    Each template is associated with a unique identifier, enclosed by line comments that delineate the
    template's start and end. A sample input format is shown below:

    Templates:

    // 1
    x = someMethod()
    // x

    Edges:

    // 1 -> 2
    // 1 -> 3

    :param source_tree: The AST containing source templates.
    :type source_tree: Tree

    :param target_tree: The AST containing target templates.
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

    def parse_templates(self) -> Dict[str, Tuple[List[Node], List[Node]]]:
        """
        Executes the actual tree traversal on both 'source_tree' and 'target_tree'. It finds corresponding template pairs
        using identifiers specified in the comments. It also finds the edges between the templates. This method
        returns a dictionary of matched template pairs, which serves as a foundation for subsequent rule inference.

        :return: A dictionary mapping identifiers to matched template pairs.
        :rtype: Dict[str, Tuple[List[Node], List[Node]]]
        """

        source_dict = self._traverse_tree(self.source_tree)
        target_dict = self._traverse_tree(self.target_tree)

        matching_pairs = {}

        for comment in source_dict:
            matching_pairs[comment] = (
                source_dict[comment],
                target_dict.get(comment, []),
            )

        return matching_pairs

    def _traverse_tree(self, tree: Tree) -> Dict[str, List[Node]]:
        """
        Performs a depth-first search (DFS) on the given tree to find nodes that are used for templates.

        :param tree: The tree to be traversed.
        :type tree: Tree
        :return: A dictionary mapping template identifiers to corresponding nodes.
        :rtype: Dict[str, List[Node]]
        """
        root = tree.root_node
        stack: Deque = deque([root])
        replacement_pairs = []
        node_start = root

        while stack:
            node = stack.pop()

            if "comment" in node.type:
                node_text = node.text.decode("utf8")
                if "->" in node_text:
                    x, y = map(str.strip, node_text[2:].split("->"))
                    self.edges[x].add(y)
                elif node_text.strip().endswith("end"):
                    replacement_pairs.append((node_start, node))
                else:
                    node_start = node

            for child in reversed(node.children):
                stack.append(child)

        replacement_dict = defaultdict(list)
        for start, end in replacement_pairs:
            begin_capture = start.end_byte
            end_capture = end.start_byte

            nodes = NodeUtils.get_nodes_in_range(root, begin_capture, end_capture)
            comment = start.text.decode("utf8")[2:].strip()
            replacement_dict[comment] = nodes

        return replacement_dict
