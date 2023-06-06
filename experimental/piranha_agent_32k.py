import os
from typing import List, Any, Optional
import attr
import openai
import time

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

prompt = '' # Your prompt goes here
print(agent.get_completion(prompt))
