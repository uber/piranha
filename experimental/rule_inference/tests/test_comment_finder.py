from unittest.mock import MagicMock, Mock

from tree_sitter import Tree

from experimental.rule_inference.comment_finder import CommentFinder


def setup_comment_finder(mock_tree_source, mock_tree_target):
    # Initialize the CommentFinder with mock trees
    comment_finder = CommentFinder(
        source_tree=mock_tree_source, target_tree=mock_tree_target
    )

    return comment_finder


def test_process_trees():
    # Mock trees
    mock_tree_source = Mock(spec=Tree)
    mock_tree_target = Mock(spec=Tree)

    # Create a CommentFinder instance
    comment_finder = setup_comment_finder(mock_tree_source, mock_tree_target)

    # Mock the _traverse_tree return values
    mock_source_dict = {"mock_comment": ["mock_node"]}
    mock_target_dict = {"mock_comment": ["mock_node"]}
    comment_finder._traverse_tree = Mock(
        side_effect=[mock_source_dict, mock_target_dict]
    )

    result = comment_finder.process_trees()

    # Assertions
    expected_result = {"mock_comment": (["mock_node"], ["mock_node"])}
    assert result == expected_result
    assert comment_finder._traverse_tree.call_count == 2


def test_traverse_tree():
    # Mock tree
    mock_tree = MagicMock(spec=Tree)

    # Create a CommentFinder instance with mock trees
    comment_finder = setup_comment_finder(mock_tree, mock_tree)

    # Mock the behaviour for a comment node
    mock_node_comment = MagicMock()
    mock_node_comment.type = "comment"
    mock_node_comment.text = MagicMock()
    mock_node_comment.text.decode.return_value = "// mock_comment"
    mock_node_comment.children = []  # Add children attribute

    # Mock the behaviour for a regular node
    mock_node = MagicMock()
    mock_node.type = "mock_type"
    mock_node.children = []  # Add children attribute

    # Mock the root_node and its properties for the tree
    mock_root_node = MagicMock()
    mock_root_node.children = [mock_node_comment, mock_node]  # Set children
    mock_tree.root_node = mock_root_node

    # Invoke the method
    result = comment_finder._traverse_tree(mock_tree)

    # Assertions
    expected_result = {"mock_comment": [mock_node]}
    assert result == expected_result
