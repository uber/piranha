from pathlib import Path
from typing import List, Tuple
from tree_sitter_languages import get_parser
from rule_inference.utils import get_patches_content, get_nodes_from_patch
import difflib
import pytest


@pytest.fixture
def load_files() -> Tuple[str, str]:
    """
    Load 'before' and 'after' java files from the specified directory.
    """
    code = Path("./tests/test_resources/demo/before.java").read_text()
    after = Path("./tests/test_resources/demo/after.java").read_text()
    return code, after


def test_node_extraction(code, after):
    """
    Main function to execute the script.
    """
    parser = get_parser("java")

    # Get unified diff
    diff = "\n".join(list(difflib.unified_diff(code.splitlines(), after.splitlines())))

    # Parse the code
    print(diff)
    tree = parser.parse(bytes(code, "utf8"))
    patches = get_patches_content(diff)

    for patch in patches:
        nodes = get_nodes_from_patch(patch, tree)
        [print(node.sexp()) for node in nodes]
        # do something with nodes...
