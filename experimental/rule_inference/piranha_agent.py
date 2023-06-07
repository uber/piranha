import os
import time
import attr
import openai
import re
import toml
import argparse
import re
import difflib
from typing import List, Any, Optional
from tree_sitter import Node
from tree_sitter_languages import get_language, get_parser
from base_prompt import BasePrompt
from polyglot_piranha import Rule, PiranhaArguments, RuleGraph, Filter, execute_piranha


class PiranhaAgentError(Exception):
    pass


@attr.s
class PiranhaAgent:
    """
    This class defines an agent that uses OpenAI's chat models for inferring Piranha rules.
    The agent takes pairs of source and target codes, finds a transformation rule between them,
    and validates the rule's effectiveness by testing if the rule can refactor the source code into the target code.
    """

    source_code = attr.ib(default="")
    target_code = attr.ib(default="")
    language = attr.ib(default="java")
    temperature = attr.ib(default=0)

    @staticmethod
    def get_tree_from_code(code: str, language: str) -> str:
        parser = get_parser(language)
        tree = parser.parse(bytes(code, "utf8"))
        root_node: Node = tree.root_node
        return root_node.sexp()

    def get_completion(self, messages, model="gpt-4-32k") -> Optional[str]:
        while True:
            try:
                response = openai.ChatCompletion.create(
                    model=model,
                    messages=messages,
                    temperature=self.temperature,  # this is the degree of randomness of the model's output
                )
                return response.choices[0].message["content"]
            except (
                openai.error.RateLimitError,
                openai.error.Timeout,
                openai.error.APIError,
            ):
                sleep_time = 10
                print(f"Rate limit reached. Sleeping for {sleep_time}s.")
                time.sleep(sleep_time)

    def infer_rules(self) -> Optional[str]:
        """Implements the inference process of the Piranha Agent.
        The function communicates with the AI model to generate a potential refactoring rule, and subsequently tests it.
        If the rule transforms the source code into the target code, the rule is returned.

        :return: str, string containing the rule in TOML format
        """
        source_tree = self.get_tree_from_code(self.source_code, self.language)
        target_tree = self.get_tree_from_code(self.target_code, self.language)
        # create diff between source and target code using difflib
        diff = difflib.unified_diff(self.source_code, self.target_code)
        diff = "\n".join(diff)

        messages = BasePrompt.generate_prompt(
            source_code=self.source_code,
            target_code=self.target_code,
            source_tree=source_tree,
            target_tree=target_tree,
            diff=diff
        )
        completion = self.get_completion(messages)
        # Define regex pattern for ```toml block
        pattern = r"```toml(.*?)```"
        # Extract all toml block contents
        toml_blocks = re.findall(pattern, completion, re.DOTALL)
        if not toml_blocks:
            raise PiranhaAgentError(
                "Could not create Piranha rule. The agent returned no TOML blocks."
            )

        toml_block = toml_blocks[0]
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
                "Piranha failed to generate the correct refactored code. The generated rule is incorrect."
            )

        return completion

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

        output_summaries = execute_piranha(args)
        return output_summaries

    @staticmethod
    def normalize_code(code: str) -> str:
        """Eliminates unnecessary spaces and newline characters from code.
        This function is as preprocessing step before comparing the refactored code with the target code.

        :param code: str, Code to normalize.
        :return: str, Normalized code.
        """

        # replace multiple spaces with a single space
        code = re.sub(r"\s+", " ", code)
        # replace multiple newlines with a single newline
        code = re.sub(r"\n+", "\n", code)
        # remove spaces before and after newlines
        code = re.sub(r" ?\n ?", "\n", code)
        # remove spaces at the beginning and end of the code
        code = code.strip()
        return code


def main():
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
    args = arg_parser.parse_args()
    source_code = open(args.source_file, "r").read()
    target_code = open(args.target_file, "r").read()

    openai.api_key = args.openai_api_key
    agent = PiranhaAgent(source_code, target_code)
    print(agent.infer_rules())


main()
