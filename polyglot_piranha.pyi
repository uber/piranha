# Copyright (c) 2022 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

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

def run_piranha_cli(path_to_codebase: str, path_to_configurations: str, dry_run: bool) -> list[PiranhaOutputSummary]:
    """
    [THIS FUNCTION IS DEPRECATED SINCE 0.2.2]
    Executes piranha for the provided configuration at {path_to_configurations} upon the given {path_to_codebase}.

    Parameters
    ------------
        path_to_codebase (str): Path to the root of the code base that Piranha will update
        path_to_configurations (str): Path to the directory that contains - `piranha_arguments.toml`, `rules.toml` and optionally `edges.toml`
        dry_run (bool): determines if Piranha should actually update the code.

    Returns:
    --------------
        list[PiranhaOutputSummary]:  Piranha Output Summary for each file touched or analyzed by Piranha. 
        For each file, it reports its content after the rewrite, the list of matches and the list of rewrites.
    """
    ...


class PiranhaArguments:
    """
    A class to capture Piranha's configurations
    """

    def __init__(
        self,
        path_to_codebase: str,
        path_to_configuration: str,
        language: str,
        substitutions: dict,
        **kwargs
    ):
        """
        Constructs `PiranhaArguments`

        Parameters
        ------------
            path_to_codebase: str
                Path to source code folder or file
            path_to_configurations: str
                 Directory containing the configuration files - `piranha_arguments.toml`, `rules.toml`, and  `edges.toml` (optional)
            language: str
                the target language
            substitutions: dict
                 Substitutions to instantiate the initial set of rules
            keyword arguments: _
                 dry_run (bool) : Disables in-place rewriting of code
                 cleanup_comments (bool) : Enables deletion of associated comments
                 cleanup_comments_buffer (int): The number of lines to consider for cleaning up the comments
                 number_of_ancestors_in_parent_scope (int): The number of ancestors considered when PARENT rules
                 delete_file_if_empty (bool): User option that determines whether an empty file will be deleted
                 delete_consecutive_new_lines (bool) : Replaces consecutive \ns  with a \n
        """
        ...

class PiranhaOutputSummary:
    """
    A class to represent Piranha's output

    Attributes
    ----------
    path : path to the file
    content : content of the file after all the rewrites
    matches : All the occurrences of "match-only" rules
    rewrites: All the applied edits
    """

    path: str
    "path to the file"

    content: str
    "content of the file after all the rewrites"

    matches: list[tuple[str, Match]]
    'All the occurrences of "match-only" rules'

    rewrites: list[Edit]
    "All the applied edits"

class Edit:
    """
     A class to represent an edit performed by Piranha

    Attributes
    ----------
    p_match : The match representing the target site of the edit
    replacement_string : The string to replace the substring encompassed by the match
    matched_rule : The rule used for creating this match-replace
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
    matched_sting : Code snippet that matched
    range : Range of the entire AST node captured by the match
    matches : The mapping between tags and string representation of the AST captured
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
