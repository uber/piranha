import os
import time
import attr
import openai
import re
import toml
import argparse
import re
from typing import List, Any, Optional, Tuple
import difflib
import logging
from logger_formatter import CustomFormatter
from tree_sitter import Node, TreeCursor
from tree_sitter_languages import get_language, get_parser
from piranha_chat import PiranhaGPTChat
from polyglot_piranha import Rule, PiranhaArguments, RuleGraph, Filter, execute_piranha


logger = logging.getLogger("PiranhaChat")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


class PiranhaAgentError(Exception):
    pass


@attr.s
class PiranhaAgent:
    """
    This class defines an agent that uses OpenAI's chat models for inferring Piranha rules.
    The agent takes pairs of source and target codes, finds a transformation rule between them,
    and validates the rule's effectiveness by testing if the rule can refactor the source code into the target code.
    """

    source_code = attr.ib(type=str)
    target_code = attr.ib(type=str)
    language = attr.ib(default="java")
    hints = attr.ib(default="")
    language_mappings = {
        "java": "java",
        "kt": "kotlin",
    }  # This is necessary because get_parser and piranha expect different naming conventions

    def to_sexp(self, node: Node, depth, prefix=""):
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
                s_exp += self.to_sexp(cursor.node, depth + 1, prefix)
            next_child = cursor.goto_next_sibling()
        return s_exp + ")"

    def get_tree_from_code(self, code: str) -> str:
        tree_sitter_language = self.language_mappings.get(self.language, self.language)
        parser = get_parser(tree_sitter_language)
        tree = parser.parse(bytes(code, "utf8"))
        root_node: Node = tree.root_node
        return self.to_sexp(root_node, 0)

    def infer_rules(self) -> Optional[Tuple[str, str]]:
        """Implements the inference process of the Piranha Agent.
        The function communicates with the AI model to generate a potential refactoring rule, and subsequently tests it.
        If the rule transforms the source code into the target code, the rule is returned.

        :return: str, string containing the rule in TOML format
        """
        source_tree = self.get_tree_from_code(self.source_code)

        # Create diff between source and target code using difflib
        diff = list(
            difflib.unified_diff(
                self.source_code.splitlines(), self.target_code.splitlines()
            )
        )
        diff = "\n".join(diff)

        prompt_holes = {
            "source_code": self.source_code,
            "source_tree": source_tree,
            "diff": diff,
            "hints": self.hints,
        }

        # Number of Chat interactions to have with the model
        n_samples = 5
        chat_interactions = [
            PiranhaGPTChat(holes=prompt_holes) for _ in range(n_samples)
        ]
        first_round = chat_interactions[0].get_completion(n_samples=n_samples)
        for i, response in enumerate(first_round):
            # Hack to prevent running the prompt multiple times (it's the same for all samples)
            # It is cheaper just to sample OpenAI API
            chat_interactions[i].append_system_message(response)

        # For each completion try to transform the source code into the target code
        max_rounds = 5
        for chat in chat_interactions:
            for i in range(max_rounds):
                try:
                    response = chat.get_model_response()
                    file_name, toml_block = self.validate_rule(response)
                    return file_name, toml_block
                except PiranhaAgentError as e:
                    # prompt_generator.append_followup(messages, e.args[0])
                    logger.debug(
                        f"GPT-4 failed to generate a rule. Following up the next round with {e}. Trying again...\n"
                    )
                    chat.append_user_followup(str(e))

        # Throw an error if no rule was found
        raise PiranhaAgentError(
            "GPT-4 failed to generate a rule. Try increasing the temperature."
        )

    def validate_rule(self, completion):
        # Define regex pattern for ```toml block
        pattern = r"```toml(.*?)```"
        # Extract all toml block contents
        toml_blocks = re.findall(pattern, completion, re.DOTALL)
        if not toml_blocks:
            raise PiranhaAgentError(
                "Could not create Piranha rule. There is no TOML block. "
                "Please return create a rule to refactor the code."
            )
        toml_block = toml_blocks[0]
        logger.debug(f"Generated rule: {toml_block}")
        toml_dict = toml.loads(toml_block)
        piranha_summary = self.run_piranha(toml_dict)
        if not piranha_summary:
            raise PiranhaAgentError(
                "Piranha did not generate any refactored code. Either the query or the filters are incorrect."
            )
        refactored_code = piranha_summary[0].content
        if self.normalize_code(refactored_code) != self.normalize_code(
            self.target_code
        ):
            raise PiranhaAgentError(
                f"The rule produced the following refactored code:\n{refactored_code}\n\n"
                f"... but the target code is:\n{self.target_code}"
            )
        pattern = r"<file_name_start>(.*?)<file_name_end>"
        file_names = re.findall(pattern, completion, re.DOTALL)
        file_name = file_names[0] if file_names else "rule.toml"
        return file_name, toml_block

    def run_piranha(self, toml_dict):
        """Runs the inferred rule by applying it to the source code using the Piranha.

        :param toml_dict: dict, Inferred rule in TOML format.
        :return: list, Summaries of the results of the execution of Piranha.
        """
        rules = toml_dict.get("rules", [])
        if not rules:
            raise PiranhaAgentError("TOML does not include any rule specifications.")
        try:
            toml_rule = rules[0]
            rule = Rule(
                name=toml_rule["name"],
                query=toml_rule["query"],
                replace_node=toml_rule["replace_node"],
                replace=toml_rule["replace"],
            )

            rule_graph = RuleGraph(rules=[rule], edges=[])
            args = PiranhaArguments(
                code_snippet=self.source_code,
                language=self.language,
                rule_graph=rule_graph,
                dry_run=True,
            )
            output_summaries = execute_piranha(args)
        except BaseException as e:
            raise PiranhaAgentError(f"Piranha failed to execute: {e}.")
        return output_summaries

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
