from pathlib import Path
from typing import List, Tuple, Dict
from tree_sitter import Language, Parser, Tree, Node, TreeCursor
from tree_sitter_languages import get_language, get_parser
import difflib
import re


def _split_file_patches(file_patch: str) -> List[str]:
    """
    Split the given file patch using a specified delimiter regex pattern.
    """
    delimiter = re.compile("@@ -[0-9,]+ \+[0-9,]+ @@.*\n")
    patches = delimiter.split(file_patch)[1:]
    delimiters = delimiter.findall(file_patch)
    patches = list(map(lambda x, y: y + x, patches, delimiters))
    return patches


def find_smallest_node(node: Node, line: str):
    """
    Find the smallest node that contains the whole line, recursively.
    """
    if node.text.decode("utf8") == line:
        return node
    else:
        for child in node.children:
            if line in child.text.decode("utf8"):
                return find_smallest_node(child, line)
    if line in node.text.decode("utf8"):
        return node

    return None


def get_deleted_lines_and_corresponding_nodes(
    patch: Tuple[int, int, str], tree: Tree
) -> Dict[str, Node]:
    """
    Return the affected nodes from the patch based on the provided tree and corresponding lines.
    """
    start_l, size, diff_content = patch

    affected_nodes = {}
    line_n = start_l

    for line in diff_content.splitlines():
        if line.startswith("-"):
            # get col number by counting the number of spaces
            # before the first non-space character
            col_n = len(line) - len(line[1:].lstrip(" ")) - 1

            # Get the node associated with this line
            start = (line_n - 1, col_n)
            end = (line_n - 1, len(line) - 1)
            node = tree.root_node.descendant_for_point_range(start, end)

            # find the smallest node that contains the whole line, recursively
            node = find_smallest_node(node, line[1:].strip())

            if line[1:].strip() != "":
                affected_nodes[line] = node

        elif line.startswith("+"):
            continue
        line_n += 1

    return affected_nodes


def get_patches_content(multiple_diffs: str) -> List[Tuple[int, int, str]]:
    """
    Extract patches content from the multiple diffs string.
    """
    multiple_diffs_sep = _split_file_patches(multiple_diffs)
    pattern = re.compile(
        r"@@ -(?P<before>[0-9,]+) \+(?P<after>[0-9,]+) @@\n\n(?P<diff_content>(.|\n)*)"
    )
    patch_content = []

    for file_diff in multiple_diffs_sep:
        for match in pattern.finditer(file_diff):
            start_l = int(match.group("before").split(",")[0])
            size = int(match.group("before").split(",")[0])
            diff_content = match.group("diff_content")

            patch_content.append((start_l, size, diff_content))

    return patch_content


def to_sexp(node: Node, depth, prefix=""):
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
            s_exp += to_sexp(cursor.node, depth + 1, prefix)
        next_child = cursor.goto_next_sibling()
    return s_exp + ")"


def to_sexp_with_str(node: Node, depth, prefix=""):
    indent = " " * depth
    cursor: TreeCursor = node.walk()
    s_exp = indent + f"{prefix}({node.type} "

    next_child = cursor.goto_first_child()
    if not next_child:
        s_exp += f") // string representation: {node.text.decode('utf8')}"
        return s_exp

    while next_child:
        child_node: Node = cursor.node
        if child_node.is_named:
            s_exp += "\n"
            prefix = ""
            if cursor.current_field_name():
                prefix = f"{cursor.current_field_name()}: "
            s_exp += to_sexp_with_str(cursor.node, depth + 1, prefix)
        next_child = cursor.goto_next_sibling()
    return s_exp + ")"


def to_source(node: Node, depth):
    cursor: TreeCursor = node.walk()
    s_exp = ""
    next_child = cursor.goto_first_child()
    if not next_child:
        s_exp += node.text.decode("utf8")
        return s_exp

    while next_child:
        s_exp += to_source(cursor.node, depth + 1) + "\n"
        next_child = cursor.goto_next_sibling()
    return s_exp
