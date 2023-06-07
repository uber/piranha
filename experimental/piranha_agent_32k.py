import os
from typing import List, Any, Optional
import attr
import openai
import time
from tree_sitter_languages import get_language, get_parser
from tree_sitter import Node
import argparse

language = get_language('python')
parser = get_parser('python')

openai.api_key = os.getenv("OPENAI_API_KEY")


@attr.s
class PiranhaAgent:
    task_explanation = attr.ib(default='')
    task_examples = attr.ib(default='')
    task_input_template = attr.ib(default='')

    def format_messages(self, **kwargs) -> str:
        formatted = self.task_explanation + "\n" + self.task_examples + "\n" + self.task_input_template.format(**kwargs)
        return formatted

    def get_completion(self, prompt: str, model="gpt-4-32k") -> Optional[str]:
        while True:
            try:
                response = openai.ChatCompletion.create(
                    model=model,
                    messages=[{"role": "user", "content": prompt}],
                    temperature=0.2,  # this is the degree of randomness of the model's output
                )
                return response.choices[0].message["content"]
            except (openai.error.RateLimitError, openai.error.Timeout, openai.error.APIError):
                sleep_time = 10
                print(f"Rate limit reached. Sleeping for {sleep_time}s.")
                time.sleep(sleep_time)


def get_tree_from_code(code: str, language: str) -> str:
    parser = get_parser(language)
    tree = parser.parse(bytes(code, "utf8"))
    root_node: Node = tree.root_node
    return root_node.sexp()

def infer_rules(**kwargs):
    path_to_explanation = "./explanation.txt"
    path_to_examples_rules = "../src/cleanup_rules"
    path_to_input_template = "./input_template.txt"

    # Get the prompt with the task explanation
    task_explanation = open(path_to_explanation, "r").read()

    # Get the example rules
    task_examples = ''
    for root, dirs, files in os.walk(path_to_examples_rules):
        for file in files:
            if file.endswith(".toml"):
                with open(os.path.join(root, file), "r") as f:
                    file_name = os.path.join(root, file)
                    file_contents = f.read()
                    file_contents = "\n".join([line for line in file_contents.split("\n") if not line.startswith("#")])
                    task_examples += f"<file_name_start> {file_name} <file_name_end>\n"
                    task_examples += f"```toml {file_contents}```\n"

    task_input = open(path_to_input_template, "r").read()
    agent = PiranhaAgent(task_explanation = task_explanation,
                         task_examples = task_examples,
                         task_input_template = task_input)

    prompt = agent.format_messages(**kwargs)
    print(agent.get_completion(prompt))

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

    kwags = { 'original_code': source_code,
                'original_tree': source_tree,
                'refactored_code': target_code,
                'refactored_tree': target_tree }


    infer_rules(**kwags)


main()