import re
from collections import defaultdict
from typing import Dict, List, Optional, Tuple

import attr
from polyglot_piranha import PiranhaArguments, Rule, RuleGraph, execute_piranha
from tree_sitter import Tree
from tree_sitter_languages import get_language, get_parser

WILDCARD = "_"


@attr.s
class TemplateParser:
    """This class provides parsing utilities for piranha's template language"""

    language = attr.ib(default="java")
    tree_sitter_language = attr.ib(default=None)
    parser = attr.ib(default=None)
    language_mappings = {
        "java": "java",
        "kt": "kotlin",
    }  # This is necessary because get_parser and piranha expect different naming conventions
    comments_node_names = {
        "java": "line_comment",
        "go": "comment",
        "kt": "comment",
        "swift": "comment",
    }
    template_holes = attr.ib(default=attr.Factory(dict))

    def __attrs_post_init__(self):
        """
        Initialize parser and language attributes for the given language after the agent object is created.
        """
        self.tree_sitter_language = get_language(
            self.language_mappings.get(self.language, self.language)
        )
        self.parser = get_parser(
            self.language_mappings.get(self.language, self.language)
        )

    def get_tree_from_code(self, code: str, remove_comments: bool = False) -> Tree:
        """
        Parse the given code and return its abstract syntax tree (AST).

        :param code: The source code to parse
        :param remove_comments: Whether to remove comments from the code before parsing
        :return: AST of the source code
        """
        code = self.replace_template_holes(code)
        if remove_comments:
            code = self.remove_comments_from_code(code)
        tree = self.parser.parse(bytes(code, "utf8"))
        return tree

    def remove_comments_from_code(self, code: str) -> str:
        """
        Removes all comments from the given code using Piranha.

        :param code: The source code from which to remove comments
        :type code: str
        :return: Source code without comments
        :rtype: str
        """
        rule = Rule(
            name="remove_comments",
            query=f"({self.comments_node_names[self.language]}) @comment",
            replace_node="comment",
            replace="",
        )
        graph = RuleGraph(rules=[rule], edges=[])
        args = PiranhaArguments(
            code_snippet=code,
            language=self.language,
            rule_graph=graph,
            dry_run=True,
        )
        output_summaries = execute_piranha(args)
        if output_summaries:
            return output_summaries[0].content
        return code

    def replace_template_holes(self, code: str) -> str:
        """
        Replace the template holes in source and target with identifiers that can be parsed by tree sitter.

        :param code: The source code to parse
        :return: mapping of template holes to identifiers, and corresponding types
        """

        template_pattern = re.compile(r":\[(?P<content>[^[]*)\]")
        matches = template_pattern.finditer(code)
        template_holes = {}
        replaced_code = code
        for match in matches:
            name, alternations = self.parse_content(match.group("content"))
            template_holes[name] = alternations or [WILDCARD]
            # replace match with identifier
            replaced_code = replaced_code.replace(match.group(), name)

        self.template_holes.update(template_holes)
        return replaced_code

    @staticmethod
    def parse_content(content: str) -> Tuple[str, Optional[List[str]]]:
        if ":" in content:
            name, alternations = [x.strip() for x in content.split(":", 1)]
            alternations = [x.strip() for x in alternations.split(",")]
            return name, alternations
        else:
            return content, None
