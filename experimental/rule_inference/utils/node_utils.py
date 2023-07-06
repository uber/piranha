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

import re
from typing import List

from tree_sitter import Node, TreeCursor


class NodeUtils:
    """
    NodeUtils is a utility class that provides static methods for performing operations on AST nodes.
    The methods include generating s-expressions, converting nodes to source code, getting non-overlapping nodes,
    removing partial nodes, and more.
    """

    @staticmethod
    def generate_sexpr(node: Node, depth: int = 0, prefix: str = "") -> str:
        """
        Creates a pretty s-expression representation of a given node.

        :param node: Node to generate the s-expression for.
        :param depth: Depth of the node in the AST.
        :param prefix: Prefix string to be appended at the start of the s-expression.
        :return: The generated s-expression.
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
                s_exp += NodeUtils.generate_sexpr(child_node, depth + 1, prefix)
            elif cursor.current_field_name():
                s_exp += "\n" + " " * (depth + 1)
                s_exp += f'{cursor.current_field_name()}: ("{child_node.type}")'
            next_child = cursor.goto_next_sibling()
        return s_exp + ")"

    @staticmethod
    def convert_to_source(
        node: Node, depth: int = 0, exclude: List[Node] = None
    ) -> str:
        """
        Convert a given node to its source code representation (unified).

        :param node: Node to convert.
        :param depth: Depth of the node in the AST.
        :param exclude: List of nodes to be excluded from the source code.
        :return: Source code representation of the node.
        """
        if exclude is None:
            exclude = []
        for to_exclude in exclude:
            if NodeUtils.contains(to_exclude, node):
                return "{placeholder}"

        cursor: TreeCursor = node.walk()
        s_exp = ""
        has_next_child = cursor.goto_first_child()
        if not has_next_child:
            s_exp += node.text.decode("utf8")
            return s_exp

        while has_next_child:
            nxt = NodeUtils.convert_to_source(cursor.node, depth + 1, exclude)
            s_exp += nxt + " "
            has_next_child = cursor.goto_next_sibling()
        return s_exp.strip()

    @staticmethod
    def get_smallest_nonoverlapping_set(nodes: List[Node]) -> List[Node]:
        """
        Get the smallest non-overlapping set of nodes from the given list.

        :param nodes: List of nodes.
        :return: The smallest non-overlapping set of nodes.
        """
        nodes = sorted(
            nodes, key=lambda x: (x.start_point, tuple(map(lambda n: -n, x.end_point)))
        )
        # get the smallest non overlapping set of nodes
        smallest_non_overlapping_set = []
        for node in nodes:
            if not smallest_non_overlapping_set:
                smallest_non_overlapping_set.append(node)
            else:
                if node.start_point > smallest_non_overlapping_set[-1].end_point:
                    smallest_non_overlapping_set.append(node)
        return smallest_non_overlapping_set

    @staticmethod
    def remove_partial_nodes(nodes: List[Node]) -> List[Node]:
        """
        Remove nodes that whose children are not contained in the replacement pair.
        Until a fixed point is reached where no more nodes can be removed.

        :param nodes: List of nodes.
        :return: The updated list of nodes after removing partial nodes.
        """
        while True:
            new_nodes = [
                node
                for node in nodes
                if all(child in node.children for child in node.children)
            ]
            if len(new_nodes) == len(nodes):
                break
            nodes = new_nodes
        return new_nodes

    @staticmethod
    def normalize_code(code: str) -> str:
        """
        Eliminates unnecessary spaces and newline characters from code.
        This function is as preprocessing step before comparing the refactored code with the target code.

        :param code: str, Code to normalize.
        :return: str, Normalized code.
        """

        # replace multiple spaces with a single space
        code = re.sub(r"\s+", "", code)
        # replace multiple newlines with a single newline
        code = re.sub(r"\n+", "", code)
        # remove spaces before and after newlines
        code = re.sub(r" ?\n ?", "", code)
        # remove spaces at the beginning and end of the code
        code = code.strip()
        return code

    @staticmethod
    def contains(node: Node, other: Node) -> bool:
        """
        Checks if the given node contains the other node.

        :param node: Node, Node to check if it contains the other node.
        :param other: Node, Node to check if it is contained by the other node.
        :return: bool, True if the given node contains the other node, False otherwise.
        """
        return (
            node.start_point <= other.start_point and node.end_point >= other.end_point
        )

    @staticmethod
    def find_lowest_common_ancestor(nodes: List[Node]) -> Node:
        """
        Find the smallest common ancestor of the provided nodes.

        :param nodes: list of nodes for which to find the smallest common ancestor.
        :return: Node which is the smallest common ancestor.
        """
        # Ensure the list of nodes isn't empty
        assert len(nodes) > 0

        # Prepare a dictionary to map node's id to the node object
        ids_to_nodes = {node.id: node for node in nodes}

        # For each node, follow its parent chain and add each one to the ancestor set and ids_to_nodes map
        ancestor_ids = [set() for _ in nodes]
        for i, node in enumerate(nodes):
            while node is not None:
                ancestor_ids[i].add(node.id)
                ids_to_nodes[node.id] = node
                node = node.parent

        # Get the intersection of all ancestor sets
        common_ancestors_ids = set.intersection(*ancestor_ids)

        # If there are no common ancestors, there's a problem with the input tree
        if not common_ancestors_ids:
            raise ValueError("Nodes have no common ancestor")

        # The LCA is the deepest node, i.e. the one with maximum start_byte
        max_start_byte_id = max(
            common_ancestors_ids, key=lambda node_id: ids_to_nodes[node_id].start_byte
        )

        return ids_to_nodes[max_start_byte_id]
