# Copyright (c) 2023 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

import logging
import os
import time
from pathlib import Path
from typing import List, Optional, Tuple

import attr
import openai

from piranha_playground.rule_inference.utils.logger_formatter import \
    CustomFormatter

logger = logging.getLogger("PiranhaAgent")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


class PiranhaChatException(Exception):
    pass


@attr.s
class PiranhaGPTChat:
    """
    A class to manage and interact with OpenAI ChatModels to generate and improve Piranha rule graphs.
    """

    explanation = '''
Your task is to improve refactoring rules for Polyglot Piranha, a tool that uses tree-sitter for parsing and refactoring code.
The rules are expressed in a domain-specific language (DSL) specific to Polyglot Piranha. Examples and explanations of the DSL will be provided below.

You will be provided with the original and refactored code snippets, along with statically inferred rules verified by an algorithm. 
However, these rules may appear unnatural since they were automatically generated. Your goal is to make them resemble rules written by humans

Key requirements:
    - The semantics of the rules should remain unchanged unless a specific request to alter them is made.
    - Strive to simplify the rules as much as possible. Always simplify lengthy s-expressions.
    - Explain each rule individually. Explain the rule in a way that a human can understand it.

Please structure your response using toml and markdown format. Refer to the expected output format below, as it will be parsed automatically.

========================= Piranha Rule Graph =========================

Piranha is language to express cascading program transformation. 
Each node in graph represents a transformation rule that identifies and modify specific code snippets. 
The edges between rules specify when, and where each rule should be applied.

========================= Piranha Rule Explanation =========================

Rules are represented in TOML. Each rule should contain at least one rule with the following properties:

- "query": Tree-sitter query to find the code pattern to refactor
- "replace_node": The captured node in the query that will be replaced
- "replace_string": Replacement string or pattern for the refactored code
- "holes": Placeholders in your queries that will be instantiated at runtime
- "is_seed_rule": Specifies whether this rule is an entry point for the rule graph.


Additionally, the rule can have optional filters. Filters can have properties like "enclosing_node", 
"not_contains", "contains", "at_least", "at_most". The filters are used to specify conditions for the rule to be applied.
       
========================= Rule Format =========================

```toml
# Define your rule within this section
[[rules]]
# Provide a unique name for your rule
name = "your_rule_name"

# Write a Tree-sitter query to identify the code pattern for refactoring. The outer most node should always be captured.
# The tree-sitter query depends on the language. The nodes you see here are for Java. You need to only use nodes in the TASK!
query = """(
    (method_invocation name: (_) @name
                       arguments: (argument_list) @args) @invk
    (#eq? @name @hole1))
"""

# Specify the captured node from the query that will be replaced
# Never add @ before the node name! Otherwise it will NOT compile!
replace_node = "invk"

# Replacement string that will substitute `replace_node`
replace = "X.other_string @args"

# Specify any placeholders in your queries that will be filled in at runtime
# In our case hole1 is used in the query, but not defined. Therefore it is a hole.
holes = ["hole1"]

# Specify if this rule should be triggered first. If it depends on other rules, set to false
is_seed_rule = true

# If necessary, define filters for your rule
[[rules.filters]]

# This pattern should match any ancestor of the captured node (optional)
enclosing_node = "(your_enclosing_node_pattern) @your_capture_name"

# Define patterns that should not be present within the enclosing_node (optional)
# Always use a list, even if you only have one pattern.
not_contains = [
    """(
    (identifier) @id
    (#eq? @id "x"))
    """,
]
# Define a pattern that should be present within the enclosing_node (optional)
contains =
    """(
    (identifier) @other_id
    (#eq? @other_id "y"))
    """
# Define the minimum and maximum number of children that should match the 'contains' pattern (optional)
at_least = 1
at_most = 5
```

========================= Edge Explanation =========================

Edges allow rules to depend on each other, thus establishing a hierarchy or sequence of application among rules. 
For instance, if a rule is defined to match a method invocation, another rule could be drafted to match a method declaration. 
In this case, the method name identified from the declaration could be utilized within the invocation.

An edge essentially describes the direction of dependency between two or more rules. It signifies that a particular rule 
('from') is based on, or derives information from, one or more other rules ('to').

Edges are also represented in the TOML format, and their structure is typically not modified unless there's a need to 
change the dependencies between rules. Your main task, unless otherwise specified, is to ensure that the 'from' and 'to' 
components of the edge correctly correspond to the names of your improved rules.

========================= Edge Format =========================

[[edges]]

# Scope of the rule - usually "Global"
scope = "Global"

# Name of the rule that depends on other rules (your rule name)
from = "your_rule_name"

# List of rules that your rule depends on (could be one or multiple)
to = ["other_rule_name", "another_rule_name"]

========================= Expected output format =========================

Your output should be a single TOML file containing the improved rules and edges, as well as an explanation in Markdown format.

Rule Graph

```toml
[[rules]] # For each rule
...

[[edges]] # For each edge
...
```


Explanation 

```md
#### `<your_rule_name1>`\n
- <Your detailed explanation>
- <Include multiple bullet points if necessary>


#### `<your_rule_name2>`\n
- <Your detailed explanation>
- <Include multiple bullet points if necessary>

```

========================= Rule Examples =========================
    '''
    input_template = """
========================= Task =========================

=== Source code === 

{source_code}

=== Tree-sitter representation (source code) ===

{source_tree}

=== Tree-sitter representation (target code) ===

{target_tree}

=== Diff === 

{diff}

=== Rules and edges to improve === 

{rules}

=== Additional requirements === 

{hints}
========================= Please simplify the rules and edges =========================

Remember, the goal is to simplify the rules and edges as much as possible while still achieving the same result.
You should only use nodes you see in the tree-sitter representation of the source code!!

    """

    add_filter_prompt = '''
Can you to further refine the following rule? Here is the request:
    
{desc}
    
========================= Current rule =========================
         
{rule}

========================= Task =========================

Improve the rule by incorporating a filter. You are permitted to add only two types of filters: enclosing_node and contains. 
You should also include an explanation for the new rule.

You're allowed to add any number of filters and further restrict the nodes using #eq, #not-eq, and #match. 


Key requirements:
    - Structure your response using TOML and Markdown formatting for automatic parsing.
    - You can ONLY chose filters from the list below. You may refine them but they should not deviate from the list.
    - Be sure to use unique names for your capture groups to avoid overlapping with existing ones from the query!
    - Make sure all the nodes are named. Every captured node should have a unique name, including the outermost node.
    - Always surround enclosing_node and contains with parenthesis (...), including the #eq, #not-eq, and #match operators.


========================= Expected output format =========================

Rules

```toml

[[rules]]
....

[[rules.filters]] # filter 1
enclosing_node = """(class_declaration) @class"""
contains = """(
    (identifier) @id (#eq? @id "x")
)"""
at_least = 1
at_most = 1

[[rules.filters]] # filter 2
enclosing_node = """(method_invocation) @invk"""

[[rules.filters]] # filter 3
enclosing_node = """(class_declaration) @class"""
contains = """(
(method_declaration
    (modifiers) @modifiers
    name: (identifier) @name) @decl
    (#eq? @name "x")
)
```

Explanation

```md
#### `<your_rule_name1>`\n
- <Your detailed explanation>
- <Include multiple bullet points if necessary>
```

========================= List of Filters =========================

=== Potential filters for enclosing node ===

{enclosing_node_filters}


"""]

========================= Errors to avoid =========================

Not surrounding the query with parenthesis (...):
enclosing_node = "(identifier) @name) (#eq? @name \"x\")"

is wrong!! it should be:
enclosing_node = """((identifier) @name) (#eq? @name "x"))"""

```
'''

    holes = attr.ib(type=dict)
    messages = attr.ib(type=list, default=attr.Factory(list))
    temperature = attr.ib(
        type=float,
        default=0.3,
        validator=[
            attr.validators.ge(0),
            attr.validators.le(1),
        ],
    )
    model = attr.ib(
        default="gpt-4-32k",
        validator=attr.validators.in_(["gpt-4", "gpt-4-32k", "gpt-3.5-turbo-16k"]),
    )

    def __attrs_post_init__(self):
        """
        Automatically called after the initialization of the instance. It gathers example rules and edge
        files from a specified path, formats the content and adds it to the internal message list.

        :param None
        :return None
        """

        examples = self._get_examples("../../src/cleanup_rules/java")

        formatted = (
            PiranhaGPTChat.explanation
            + "\n"
            + examples
            + "\n"
            + PiranhaGPTChat.input_template.format(**self.holes)
        )

        self.messages.append({"role": "user", "content": formatted})

    def append_system_message(self, system_message: str):
        """
        Appends a message from the GPT model to the internal message list.

        :param system_message: str: The message content to be added to the message list.
        :return None
        """
        self.messages.append({"role": "assistant", "content": system_message})

    def append_user_followup(self, followup_message: str):
        """
        Appends a follow-up message from the user to the internal message list.

        :param followup_message: str: The message content to be added to the message list.
        :return None
        """
        self.messages.append({"role": "user", "content": followup_message})

    def get_model_response(self) -> str:
        """
        Fetches the latest message from the GPT model. If the latest message is from the user, it will trigger
        a new GPT model prediction and append the response to the internal message list.

        :param None
        :return str: The latest message content from the GPT model.
        """
        latest_message = self.messages[-1]
        if latest_message["role"] == "assistant":
            return latest_message["content"]
        else:
            completions = self.get_completion(n_samples=1)
            content = completions[0]
            self.append_system_message(content)
            return content

    def append_improve_request(self, desc, rule, enclosing_nodes):
        """
        Appends a request to improve the rule to the internal message list.

        :param desc: str: Description of the request.
        :param rule: str: The rule to be improved.
        :param enclosing_nodes: str: The enclosing nodes to be included in the rule.
        :return None
        """

        self.messages.append(
            {
                "role": "user",
                "content": PiranhaGPTChat.add_filter_prompt.format(
                    desc=desc,
                    rule=rule,
                    enclosing_node_filters=enclosing_nodes,
                ),
            }
        )

    def get_completion(self, n_samples: int = 1) -> Optional[List[str]]:
        """
        Attempts to generate a new GPT model prediction based on the internal message list. It handles
        common OpenAI API exceptions such as rate limiting and API errors.

        :param n_samples: int: Number of samples to generate from the model.
        :return List[str]: A list of generated messages. None if an API exception occurs.
        :raises PiranhaChatException: If it fails to generate a completion from the GPT model after three attempts.
        """

        for _ in range(3):
            try:
                logger.debug("Attempting to get completion from GPT.")
                response = openai.ChatCompletion.create(
                    model=self.model,
                    messages=self.messages,
                    temperature=self.temperature,  # this is the degree of randomness of the model's output
                    n=n_samples,
                )
                return [
                    response.choices[i].message.content
                    for i in range(len(response.choices))
                ]
            except (
                openai.error.RateLimitError,
                openai.error.Timeout,
                openai.error.APIError,
            ) as e:
                logger.error(e)
                sleep_time = 0.5
                logger.error(f"Rate limit reached. Sleeping for {sleep_time}s.")
                time.sleep(sleep_time)
        raise PiranhaChatException("Failed to get completion from GPT.")

    @staticmethod
    def _get_examples(path_to_examples_rules):
        """
        Walks through a specified directory to gather and format the content of example rule and edge files.
        The formatted content is then returned as a single string.

        :param path_to_examples_rules: str: Path to the directory containing example rule and edge files.
        :return str: Formatted content of example rule and edge files.
        """

        task_examples = ""
        for root, dirs, files in os.walk(path_to_examples_rules):
            for file in files:
                if file.endswith("rules.toml") or file.endswith("edges.toml"):
                    file_name = os.path.join(root, file)
                    file_contents = Path(file_name).read_text()
                    file_contents = "\n".join(
                        [
                            line
                            for line in file_contents.split("\n")
                            if not line.startswith("#")
                        ]
                    )
                    task_examples += f"<file_name_start> {file_name} <file_name_end>\n"
                    task_examples += f"```toml {file_contents}```\n"
        return task_examples
