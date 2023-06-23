import logging
import os
from pathlib import Path
from typing import List, Optional
from experimental.rule_inference.utils.logger_formatter import CustomFormatter
import attr
import openai
import time

logger = logging.getLogger("PiranhaAgent")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


@attr.s
class PiranhaGPTChat:
    explanation = '''
Your task is to improve refactoring rules for Polyglot Piranha, a tool that uses tree-sitter for parsing and refactoring code.
The rules are expressed in a domain-specific language (DSL) specific to Polyglot Piranha. Examples and explanations of the DSL will be provided below.

You will be provided with the original and refactored code snippets, along with statically inferred rules verified by an algorithm. 
However, these rules may appear unnatural since they were automatically generated. Your goal is to make them resemble rules written by humans, 
incorporating meaningful variable names and simpler matchers. Whenever possible, simplify lengthy s-expressions. Note however, you should
keep capture groups in the "replace" string if they are meaningful (such as variable names, method names, etc.). You have to make sure you 
ONLY use nodes that you observe in the TASK! Beware tree-sitter queries are language dependent. DO NOT use any other nodes.

You should not alter the semantics of the rules without a specific request. 
However, if the user asks you to add filters or extra constraints, you should accommodate their needs. 
In such cases, please ensure that you explain your change and provide a justification for any changes made.
Additionally, always strive to simplify the rules as much as possible.

========================= Piranha Rule Explanation =========================

Rules are represented in TOML. Each rule should contain at least one rule with the following properties:

- "query": Tree-sitter query to find the code pattern to refactor
- "replace_node": The captured node in the query that will be replaced
- "replace_string": Replacement string or pattern for the refactored code
- "holes": Placeholders in your queries that will be instantiated at runtime

Additionally, the rule can have optional properties such as "is_seed_rule", "groups", and "filters".
Filters can have properties like "enclosing_node", "not_contains", "contains", "at_least", "at_most".
The filters are used to specify conditions for the rule to be applied.
       
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

Your output should be a single TOML file containing the improved rules and edges:

```toml
[[rules]] # For each rule
...

[[edges]] # For each edge
...
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
        examples = self._get_examples("../../src/cleanup_rules/java")
        examples = self._get_examples("../../src/cleanup_rules/go")
        examples = self._get_examples("../../src/cleanup_rules/swift")
        examples = self._get_examples("../../src/cleanup_rules/kt")

        formatted = (
            PiranhaGPTChat.explanation
            + "\n"
            + examples
            + "\n"
            + PiranhaGPTChat.input_template.format(**self.holes)
        )

        self.messages.append({"role": "user", "content": formatted})

    def append_system_message(self, system_message):
        """Add a GPT response to the internal messages"""
        self.messages.append({"role": "assistant", "content": system_message})

    def append_user_followup(self, followup_message):
        """Add a followup message from the user after GPT replies"""
        self.messages.append({"role": "user", "content": followup_message})

    def get_model_response(self):
        latest_message = self.messages[-1]
        if latest_message["role"] == "assistant":
            return latest_message["content"]
        else:
            completions = self.get_completion(n_samples=1)
            content = completions[0]
            self.append_system_message(content)
            return content

    def get_completion(self, n_samples: int = 1) -> Optional[List[str]]:
        while True:
            try:
                logger.debug(self.messages[-1]["content"])
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
                sleep_time = 10
                logger.error(f"Rate limit reached. Sleeping for {sleep_time}s.")
                time.sleep(sleep_time)

    @staticmethod
    def _get_examples(path_to_examples_rules):
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
