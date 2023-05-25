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

from typing import List, Optional


def execute_piranha(piranha_argument: PiranhaArguments) -> list[PiranhaOutputSummary]:
    """
    Executes piranha for the given `piranha_arguments` and returns `PiranhaOutputSummary` for each file analyzed by Piranha
    Parameters
    ------------
        piranha_arguments: Piranha Arguments
            Configurations for piranha
    Returns
    ------------
    List of `PiranhaOutPutSummary`
    """
    ...

class PiranhaArguments:
    """
    A class to capture Piranha's configurations
    """

    def __init__(
        self,
        language: str,
        path_to_codebase: Optional[str] = None,
        include: Optional[List[str]] = None,
        exclude: Optional[List[str]] = None,
        substitutions: Optional[dict] = None,
        path_to_configurations: Optional[str] = None,
        rule_graph: Optional[RuleGraph]= None,
        code_snippet: Optional[str] = None,
        dry_run: Optional[bool] = None,
        cleanup_comments: Optional[bool] = None,
        cleanup_comments_buffer: Optional[int] = None,
        number_of_ancestors_in_parent_scope: Optional[int] = None,
        delete_consecutive_new_lines: Optional[bool] = None,
        global_tag_prefix: Optional[str] = 'GLOBAL_TAG',
        delete_file_if_empty: Optional[bool] = None,
        path_to_output: Optional[str] = None,
        allow_dirty_ast: Optional[bool] = None
    ):
        """
        Constructs `PiranhaArguments`

        Parameters
        ------------
            language: str
                the target language
            path_to_codebase: str
                Path to source code folder or file
            keyword arguments: _
                 substitutions (dict): Substitutions to instantiate the initial set of rules
                 path_to_configurations (str): Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`, and  `edges.toml`
                 rule_graph (RuleGraph): The rule graph constructed via RuleGraph DSL
                 code_snippet (str): The input code snippet to transform
                 dry_run (bool): Disables in-place rewriting of code
                 cleanup_comments (bool): Enables deletion of associated comments
                 cleanup_comments_buffer (int): The number of lines to consider for cleaning up the comments
                 number_of_ancestors_in_parent_scope (int): The number of ancestors considered when PARENT rules
                 delete_consecutive_new_lines (bool): Replaces consecutive \ns  with a \n
                 global_tag_prefix (str): the prefix for global tags
                 delete_file_if_empty (bool): User option that determines whether an empty file will be deleted
                 path_to_output (str): Path to the output json file
                 allow_dirty_ast (bool): Allows syntax errors in the input source code 
        """
        ...

class PiranhaOutputSummary:
    """
    A class to represent Piranha's output

    Attributes
    ----------
    path: path to the file
    content: content of the file after all the rewrites
    matches: All the occurrences of "match-only" rules
    rewrites: All the applied edits
    """

    path: str
    "path to the file"

    original_content: str
    "Original content of the file before any rewrites"

    content: str
    "Final content of the file after all the rewrites"

    matches: list[tuple[str, Match]]
    'All the occurrences of "match-only" rules'

    rewrites: list[Edit]
    "All the applied edits"

class Edit:
    """
     A class to represent an edit performed by Piranha

    Attributes
    ----------
    p_match: The match representing the target site of the edit
    replacement_string: The string to replace the substring encompassed by the match
    matched_rule: The rule used for creating this match-replace
    """

    p_match: Match
    "The match representing the target site of the edit"

    matched_rule: str
    "The rule used for creating this match-replace"

    replacement_string: str
    "The string to replace the substring encompassed by the match"

class Match:
    """
     A class to represent a match

    Attributes
    ----------
    matched_sting: Code snippet that matched
    range: Range of the entire AST node captured by the match
    matches: The mapping between tags and string representation of the AST captured
    """

    matched_string: str
    "Code snippet that matched"

    range: Range
    "Range of the entire AST node captured by the match"

    matches: dict
    "The mapping between tags and string representation of the AST captured"
    ""

class Range:
    """A range of positions in a multi-line text document,
    both in terms of bytes and of rows and columns.
    """

    start_byte: int
    end_byte: int
    start_point: Point
    end_point: Point

class Point:
    row: int
    column: int

class Filter:
    """ A class to capture filters of a Piranha Rule
    """
    enclosing_node: TSQuery
    "AST patterns that some ancestor node of the primary match should comply"
    not_contains: list[TSQuery]
    "AST patterns that SHOULD NOT match any subtree of node matching `enclosing_node` pattern"
    contains: TSQuery
    "AST pattern that SHOULD match subtrees of `enclosing_node`. " \
    "Number of matches should be within the range of `at_least` and `at_most`."
    at_least: int
    "The minimum number of times the contains query should match in the enclosing node"
    at_most: int
    "The maximum number of times the contains query should match in the enclosing node"
    def __init__(
        self,
        enclosing_node: str = '',
        not_contains: list[str] = [],
        contains: str = '',
        at_least: int = 1,
        at_most: int = 4294967295 # u32::MAX
    ):
        """
        Constructs `Filter`

        Parameters
        ------------
            enclosing_node: str
                AST patterns that some ancestor node of the primary match should comply
            not_contains: list[str]
                 AST patterns that should not match any subtree of node matching `enclosing_node` pattern
        """
        ...

class Rule:
    """ A class to capture Piranha Rule
    """
    name: str
    "Name of the rule"
    query: TSQuery
    "Tree-sitter query as string"
    replace_node: str
    "The tag corresponding to the node to be replaced"
    replace: str
    "Replacement pattern"
    groups: set[str]
    "Group(s) to which the rule belongs"
    holes: set[str]
    "Holes that need to be filled, in order to instantiate a rule"
    filters: set[Filter]
    "Filters to test before applying a rule"

    def __init__(
        self,
        name: str,
        query: str,
        replace_node: Optional[str] = None,
        replace: Optional[str] = None,
        groups: set[str] = set(),
        holes: set[str] = set(),
        filters: set[Filter] = set(),
        is_seed_rule: bool = True,
    ):
        """
        Constructs `Rule`

        Parameters
        ------------
            name: str
                Name of the rule
            query: str
                Tree-sitter query as string
            replace_node: str
                The tag corresponding to the node to be replaced
            replace: str
                Replacement pattern
            groups: set[str]
                Group(s) to which the rule belongs
            holes: set[str]
                Holes that need to be filled, in order to instantiate a rule
            filters: set[Filter]
                Filters to test before applying a rule
        """
        ...

class OutgoingEdges:
    frm: str
    "The source rule or group of rules"
    to: list[str]
    "The target edges or groups of edges"
    scope: str
    "The scope label for the edge"

    def __init__(
        self,
        frm: str,
        to: list[str],
        scope: str,
    ):
        """
        Constructs `OutgoingEdge`

        Parameters
        ------------
            frm: str
                The source rule or group of rules
            to: list[str]
                The target edges or groups of edges
            scope: str
                The scope label for the edge
        """
        ...

class RuleGraph:
    rules: list[Rule]
    "The rules in the graph"
    edges: list[OutgoingEdges]
    "The edges in the graph"
    graph: dict
    "The graph itself (as an adjacency list)"

    def __init__(
        self,
        rules: list[Rule],
        edges: list[OutgoingEdges],
    ):
        """
        Constructs `OutgoingEdge`

        Parameters
        ------------
            rules: list[Rule]
                The rules in the graph
            edges: list[OutgoingEdges]
                The edges in the graph
        """
        ...

class TSQuery:
    "Captures a Tree sitter query"
    def query(self):
        """The query
        """
        ...
