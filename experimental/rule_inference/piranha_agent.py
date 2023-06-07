import os
import time
import attr
import openai
import re
import toml
import argparse
from typing import List, Any, Optional
from tree_sitter import Node
from tree_sitter_languages import get_language, get_parser
from base_prompt import BasePrompt
from polyglot_piranha import Rule, PiranhaArguments, RuleGraph, Filter, execute_piranha


class PiranhaAgentError(Exception):
    pass


@attr.s
class PiranhaAgent:
    source_code = attr.ib(default="")
    target_code = attr.ib(default="")
    language = attr.ib(default="java")

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
                    temperature=0.2,  # this is the degree of randomness of the model's output
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

    def infer_rules(self):
        source_tree = self.get_tree_from_code(self.source_code, self.language)
        target_tree = self.get_tree_from_code(self.target_code, self.language)

        messages = BasePrompt.generate_prompt(source_code = self.source_code,
                                              target_code = self.target_code,
                                              source_tree = source_tree,
                                              target_tree = target_tree,)
        completion = self.get_completion(messages)
        # Define regex pattern for ```toml block
        pattern = r"```toml(.*?)```"
        # Extract all toml block contents
        toml_blocks = re.findall(pattern, completion, re.DOTALL)
        if not toml_blocks:
            raise PiranhaAgentError("No TOML blocks found in the completion")

        toml_block = toml_blocks[0]
        toml_dict = toml.loads(toml_block)
        self.test_piranha(toml_dict)

        return completion

    def test_piranha(self, toml_dict):
        rules = toml_dict.get("rules", [])
        if not rules:
            raise PiranhaAgentError("No rules found in TOML")

        toml_rule = rules[0]
        rule = Rule (
            name= toml_rule["name"],
            query= toml_rule["query"],
            replace_node= toml_rule["replace_node"],
            replace= toml_rule["replace"],
        )

        rule_graph = RuleGraph(
            rules= [rule],
            edges = []
        )

        args = PiranhaArguments(
            code_snippet=self.source_code,
            language=self.language,
            rule_graph = rule_graph,
            dry_run=True,
        )

        output_summaries = execute_piranha(args)
        print(output_summaries[0].content)


def main():
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument("-s", "--source-file", type=str, required=True)
    arg_parser.add_argument("-t", "--target-file", type=str, required=True)
    arg_parser.add_argument("-l", "--language", type=str, default="java")

    args = arg_parser.parse_args()
    source_code = open(args.source_file, "r").read()
    target_code = open(args.target_file, "r").read()


    agent = PiranhaAgent(source_code, target_code)
    agent.infer_rules()


main()
