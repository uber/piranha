from tree_sitter import TreeCursor, Node
from typing import List
import re


class NodeUtils:
    @staticmethod
    def generate_sexpr(node, depth=0, prefix=""):
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
    def convert_to_source(node: Node, depth=0, exclude=None):
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

        for child in node.children:
            nxt = NodeUtils.convert_to_source(child, depth + 1, exclude)
            s_exp += nxt + " "
        return s_exp.strip()

    @staticmethod
    def get_smallest_nonoverlapping_set(nodes: List[Node]):
        """
        Get the smallest non overlapping set of nodes from the given list.
        :param nodes:
        :return:
        """
        # sort the nodes by their start position
        # if the start positions are equal, sort by end position in reverse order
        nodes = sorted(
            nodes, key=lambda x: (x.start_point, tuple(map(lambda n: -n, x.end_point)))
        )
        # get the smallest non overlapping set of nodes
        smallest_non_overlapping_set = []
        for node in nodes:
            if not smallest_non_overlapping_set:
                smallest_non_overlapping_set.append(node)
            else:
                if node.start_point >= smallest_non_overlapping_set[-1].end_point:
                    smallest_non_overlapping_set.append(node)
        return smallest_non_overlapping_set

    @staticmethod
    def remove_partial_nodes(nodes: List[Node]) -> List[Node]:
        """
        Remove nodes that whose children are not contained in the replacement pair.
        Until a fixed point is reached where no more nodes can be removed.
        """
        while True:
            new_nodes = [
                node for node in nodes if all(child in nodes for child in node.children)
            ]
            if len(new_nodes) == len(nodes):
                break
            nodes = new_nodes
        return new_nodes

    @staticmethod
    def normalize_code(code: str) -> str:
        """Eliminates unnecessary spaces and newline characters from code.
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
        """Checks if the given node contains the other node.

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
