import difflib
import re
from pathlib import Path
from typing import Dict, List, Tuple

import attr
from experimental.rule_inference.utils.node_utils import NodeUtils
from tree_sitter import Language, Node, Parser, Tree, TreeCursor
from tree_sitter_languages import get_language, get_parser


@attr.s
class Patch:
    start_line: int = attr.ib()
    line_count: int = attr.ib()
    start_line_after: int = attr.ib()
    line_count_after: int = attr.ib()
    diff_content: str = attr.ib()

    @staticmethod
    def split_patches(file_patch: str) -> List[str]:
        delimiter = re.compile("@@ -[0-9,]+ \+[0-9,]+ @@.*\n")
        patches = delimiter.split(file_patch)[1:]
        delimiters = delimiter.findall(file_patch)
        patches = list(map(lambda x, y: y + x, patches, delimiters))
        return patches

    @staticmethod
    def extract_smallest_node(node: Node, line: str):
        for child in node.children:
            if line in child.text.decode("utf8"):
                return Patch.extract_smallest_node(child, line)
        return node

    def _extract_line_info(
        self, line: str, line_n: int, tree: Tree
    ) -> Tuple[Node, int]:
        """Extract relevant information from a given line in a diff."""
        col_n = len(line) - len(line.lstrip(" "))
        start = (line_n - 1, col_n)
        end = (line_n - 1, len(line) - 1)
        node = tree.root_node.descendant_for_point_range(start, end)
        node = self.extract_smallest_node(node, line[1:].strip())
        return node, line_n + 1

    def get_nodes_from_patch(
        self, source_tree: Tree, target_tree: Tree
    ) -> List[Tuple[Dict[str, Node], Dict[str, Node]]]:
        node_pairs = []
        nodes_after = {}
        nodes_before = {}
        line_number_before = self.start_line
        line_number_after = self.start_line_after

        for line in self.diff_content.splitlines():
            if line.startswith("-") and line[1:].strip() != "":
                node_before, line_number_before = self._extract_line_info(
                    line, line_number_before, source_tree
                )
                nodes_before[line[1:]] = node_before

            elif line.startswith("+") and line[1:].strip() != "":
                node_after, line_number_after = self._extract_line_info(
                    line, line_number_after, target_tree
                )
                nodes_after[line[1:]] = node_after
            else:
                if nodes_before != {} or nodes_after != {}:
                    node_pairs.append((nodes_before, nodes_after))
                nodes_before = {}
                nodes_after = {}
                line_number_before += 1
                line_number_after += 1

        if nodes_before != {} or nodes_after != {}:
            node_pairs.append((nodes_before, nodes_after))
        return node_pairs

    @classmethod
    def from_diffs(cls, multiple_diffs: str) -> List["Patch"]:
        def extract_nums(num_str):
            nums = num_str.split(",")
            if len(nums) == 1:
                nums.append("0")
            return map(int, nums)

        diff_segments = cls.split_patches(multiple_diffs)
        pattern = re.compile(
            r"@@ -(?P<before>[0-9,]+) \+(?P<after>[0-9,]+) @@\n\n(?P<diff_content>(.|\n)*)"
        )
        patches = []

        for file_diff in diff_segments:
            for match in pattern.finditer(file_diff):
                start_line, line_count = extract_nums(match.group("before"))
                start_line_after, line_count_after = extract_nums(match.group("after"))
                diff_content = match.group("diff_content")

                patch = cls(
                    start_line,
                    line_count,
                    start_line_after,
                    line_count_after,
                    diff_content,
                )
                patches.append(patch)

        return patches
