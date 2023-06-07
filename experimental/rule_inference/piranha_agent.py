import os
from typing import List, Any, Optional
import attr
import openai
import time
from tree_sitter_languages import get_language, get_parser
from tree_sitter import Node
import argparse
from base_prompt import BasePrompt

language = get_language("python")
parser = get_parser("python")

openai.api_key = os.getenv("OPENAI_API_KEY")


@attr.s
class PiranhaAgent:

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

    def infer_rules(self, **kwargs):
        messages = BasePrompt.generate_prompt(**kwargs)
        completion = self.get_completion(messages)
        print(completion)
        return completion



def get_tree_from_code(code: str, language: str) -> str:
    parser = get_parser(language)
    tree = parser.parse(bytes(code, "utf8"))
    root_node: Node = tree.root_node
    return root_node.sexp()


def main():
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument("-s", "--source-file", type=str, required=True)
    arg_parser.add_argument("-t", "--target-file", type=str, required=True)
    arg_parser.add_argument("-l", "--language", type=str, default="java")

    args = arg_parser.parse_args()
    source_code = open(args.source_file, "r").read()
    source_tree = get_tree_from_code(source_code, args.language)
    target_code = open(args.target_file, "r").read()
    target_tree = get_tree_from_code(target_code, args.language)

    kwags = {
        "original_code": source_code,
        "original_tree": source_tree,
        "refactored_code": target_code,
        "refactored_tree": target_tree,
    }

    agent = PiranhaAgent()
    agent.infer_rules(**kwags)


main()
