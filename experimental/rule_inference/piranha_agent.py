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
from tree_sitter import Node
from tree_sitter_languages import get_language, get_parser
from piranha_chat import PiranhaGPTChat
from polyglot_piranha import Rule, PiranhaArguments, RuleGraph, Filter, execute_piranha
from rule_application import CodebaseRefactorer


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
    language_mappings = {
        "java": "java",
        "kt": "kotlin",
    }  # This is necessary because get_parser and piranha expect different naming conventions

    def get_tree_from_code(self, code: str) -> str:
        tree_sitter_language = self.language_mappings.get(self.language, self.language)
        parser = get_parser(tree_sitter_language)
        tree = parser.parse(bytes(code, "utf8"))
        root_node: Node = tree.root_node
        return root_node.sexp()

    def infer_rules(self) -> Optional[Tuple[str, str]]:
        """Implements the inference process of the Piranha Agent.
        The function communicates with the AI model to generate a potential refactoring rule, and subsequently tests it.
        If the rule transforms the source code into the target code, the rule is returned.

        :return: str, string containing the rule in TOML format
        """
        source_tree = self.get_tree_from_code(self.source_code)
        target_tree = self.get_tree_from_code(self.target_code)

        # Create diff between source and target code using difflib
        diff = list(
            difflib.unified_diff(
                self.source_code.splitlines(), self.target_code.splitlines()
            )
        )
        diff = "\n".join(diff)

        prompt_holes = {
            "source_code": self.source_code,
            "target_code": self.target_code,
            "source_tree": source_tree,
            "target_tree": target_tree,
            "diff": diff,
        }

        # Number of Chat interactions to have with the model
        n_samples = 1
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
        for i in range(max_rounds):
            for chat in chat_interactions:
                try:
                    response = chat.get_model_response()
                    file_name, toml_block = self.validate_rule(response)
                    return file_name, toml_block
                except PiranhaAgentError as e:
                    # prompt_generator.append_followup(messages, e.args[0])
                    logger.debug(
                        f"GPT-4 failed to generate a rule. Following up the next round with {e}. Trying again..."
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
                f"GPT generated a bad rule. Run again to get a new sample. Generated rule: {toml_block}\n"
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

        try:
            output_summaries = execute_piranha(args)
        except BaseException as e:
            raise PiranhaAgentError(f"Piranha failed to execute: {e}")
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


def main():
    logger.info("Starting Piranha Agent")
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "-s",
        "--source-file",
        type=str,
        required=True,
        help="Path to the original source file containing the code before the transformation",
    )
    arg_parser.add_argument(
        "-t",
        "--target-file",
        type=str,
        required=True,
        help="Path to the target source file containing the code after the transformation",
    )
    arg_parser.add_argument(
        "-l",
        "--language",
        type=str,
        default="java",
        help="Language of the source and target code",
    )
    arg_parser.add_argument(
        "-k", "--openai-api-key", type=str, required=True, help="OpenAI API key"
    )

    arg_parser.add_argument(
        "-p",
        "--path-to-codebase",
        type=str,
        default="",
        help="Code base where the rule should be applied after a successful inference",
    )

    arg_parser.add_argument(
        "-c",
        "--path-to-piranha-config",
        type=str,
        default="./piranha-configs/",
        help="The directory where rule should be persisted",
    )

    args = arg_parser.parse_args()
    source_code = open(args.source_file, "r").read()
    target_code = open(args.target_file, "r").read()

    openai.api_key = args.openai_api_key
    agent = PiranhaAgent(source_code, target_code, language=args.language)

    rule_name, rule = agent.infer_rules()
    logger.info(f"Generated rule:\n{rule}")

    file_path = os.path.join(args.path_to_piranha_config, rule_name)
    logger.info(f"Writing rule to {file_path}")
    os.makedirs(args.path_to_piranha_config, exist_ok=True)
    with open(file_path, "w") as f:
        f.write(rule)

    if args.path_to_codebase:
        refactor = CodebaseRefactorer(
            args.language, args.path_to_codebase, args.path_to_piranha_config
        )
        refactor.refactor_codebase()


main()
