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
            next_child = cursor.goto_next_sibling()
        return s_exp + ")"

    @staticmethod
    def convert_to_source(node, depth=0):
        cursor: TreeCursor = node.walk()
        s_exp = ""
        has_next_child = cursor.goto_first_child()
        if not has_next_child:
            s_exp += node.text.decode("utf8")
            return s_exp

        while has_next_child:
            s_exp += NodeUtils.convert_to_source(cursor.node, depth + 1)
            has_next_child = cursor.goto_next_sibling()
        return s_exp

    @staticmethod
    def get_smallest_nonoverlapping_set(nodes: List[Node]):
        """
        Get the smallest non overlapping set of nodes from the given list.
        :param nodes:
        :return:
        """
        # sort the nodes by their start position
        nodes = sorted(nodes, key=lambda x: x.start_point)
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
