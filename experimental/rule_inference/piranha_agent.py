import multiprocessing
import os
import time
import attr
import openai
import re
import toml
import argparse
import asyncio
import re
from typing import List, Any, Optional, Tuple
import difflib
import logging
from logger_formatter import CustomFormatter
from tree_sitter_languages import get_language, get_parser
from static_inference import Inference
from piranha_chat import PiranhaGPTChat
from polyglot_piranha import Rule, PiranhaArguments, RuleGraph, Filter, execute_piranha
from patch import Patch
from multiprocessing import Pool
from static_inference import QueryWriter
from tree_sitter import Language, Parser, Tree, Node, TreeCursor
from node_utils import NodeUtils
from comment_finder import CommentFinder

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

    def get_tree_from_code(self, code: str) -> Tree:
        tree_sitter_language = self.language_mappings.get(self.language, self.language)
        parser = get_parser(tree_sitter_language)
        tree = parser.parse(bytes(code, "utf8"))
        return tree

    def infer_rules(self, callback=None) -> Optional[Tuple[str, str]]:
        """Implements the inference process of the Piranha Agent.
        The function communicates with the AI model to generate a potential refactoring rule, and subsequently tests it.
        If the rule transforms the source code into the target code, the rule is returned.

        :return: str, string containing the rule in TOML format
        """
        source_tree = self.get_tree_from_code(self.source_code)
        target_tree = self.get_tree_from_code(self.target_code)

        source_tree_sexpr = NodeUtils.generate_sexpr(source_tree.root_node, 0)
        target_tree_sexpr = NodeUtils.generate_sexpr(target_tree.root_node, 0)
        # Create diff between source and target code using difflib
        diff = list(
            difflib.unified_diff(
                self.source_code.splitlines(), self.target_code.splitlines()
            )
        )
        diff = "\n".join(diff)
        # diff = self.append_diff_information(diff, source_tree, target_tree)

        rules = ""
        finder = CommentFinder(source_tree, target_tree)
        pairs = finder.find_replacement_pairs()
        for nodes_before, nodes_after in pairs.values():
            inference_engine = Inference(nodes_before, nodes_after)
            rules += inference_engine.static_infer() + "\n\n"

        if callback:
            callback(rules)

        prompt_holes = {
            "source_code": self.source_code,
            "source_tree": source_tree_sexpr,
            "target_tree": target_tree_sexpr,
            "diff": diff,
            "rules": rules,
            "hints": self.hints,
        }

        # Number of Chat interactions to have with the model
        n_samples = 15
        chat_interactions = [
            PiranhaGPTChat(holes=prompt_holes) for _ in range(n_samples)
        ]
        first_round = chat_interactions[0].get_completion(n_samples=n_samples)
        for i, response in enumerate(first_round):
            # Hack to prevent running the prompt multiple times (it's the same for all samples)
            # It is cheaper just to sample OpenAI API
            chat_interactions[i].append_system_message(response)

        # For each completion try to transform the source code into the target code
        max_rounds = 10
        for chat in chat_interactions:
            for i in range(max_rounds):
                try:
                    file_name, toml_block = self.validate_rule_wrapper(chat)
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

    def append_diff_information(self, diff, source_tree, target_tree):
        patches: List[Patch] = Patch.from_diffs(diff)
        # Append to the diff the information about the deleted lines for each patch
        diff += "\n=== Draft queries to represent deleted lines ===\n\n"
        for patch in patches:
            node_pairs = patch.get_nodes_from_patch(source_tree, target_tree)
            for nodes_before, nodes_after in node_pairs:
                for line, node in nodes_before.items():
                    q = QueryWriter()
                    diff += f"\n\n--------\n\nDelete Line: {line} \n\nCorresponding query:\n{q.write([node])}"
        return diff

    def validate_rule_wrapper(self, chat):
        # with Pool(processes=1) as pool:
        completion = chat.get_model_response()
        # result = pool.apply_async(self.validate_rule, (completion,))
        try:
            file_name, toml_block = self.validate_rule(completion)
            # file_name, toml_block = result.get(
            #    timeout=5
            # )  # Add a timeout of 5 seconds
            if file_name and toml_block:
                return file_name, toml_block

        except multiprocessing.context.TimeoutError:
            raise PiranhaAgentError(
                "Piranha in infinite loop. Please add a filter or constraint the query. "
                "Remember you can only constraint queries with #eq, #not-eq, #match. "
                "Otherwise you need to use a [[rules.filters]] with contains or not_contains."
            )

    def validate_rule(self, completion):
        # Define regex pattern for ```toml block
        pattern = r"```toml(.*?)```"
        # Extract all toml block contents
        toml_blocks = re.findall(pattern, completion, re.DOTALL)
        if not toml_blocks:
            raise PiranhaAgentError(
                "Could not create Piranha rule. There is no TOML block. "
                "Please create a rule to refactor the code."
            )

        try:
            toml_block = (
                toml_blocks[0].replace("parenthesized_expression", "condition").strip()
            )
            logger.debug(f"Generated rule: {toml_block}")
            toml_dict = toml.loads(toml_block)
            return "rule.toml", toml_block
        except Exception as e:
            raise PiranhaAgentError(
                f"Could not create Piranha rule. The TOML block is not valid. {e}"
            )

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
                f"The rule produced wrong code!!! "
                f"Expected:\n{self.target_code}\n\n but got:\n{refactored_code}\n\n"
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

            # get rules.filters
            filters = toml_rule.get("filters", None)
            filters_lst = set()
            if filters:
                filters = filters[0]
                # get enclosing node
                enclosing_node = filters.get("enclosing_node", "")
                # get not contains which is a list of strings
                not_contains = filters.get("not_contains", [])
                # create a filter
                filter = Filter(
                    enclosing_node=enclosing_node,
                    not_contains=not_contains,
                )

                filters_lst.add(filter)

            # Add a check to prevent recursion

            rule = Rule(
                name=toml_rule["name"],
                query=toml_rule["query"],
                replace_node=toml_rule["replace_node"],
                replace=toml_rule["replace"],
                filters=filters_lst,
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
            if "QueryError" in str(e):
                raise PiranhaAgentError(
                    f"Piranha failed to execute. The query is not valid. {e}. Make sure you are not re-using capture "
                    f"groups in contains and not_contains. You cannot reuse the same @tags you used in the query."
                )
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
