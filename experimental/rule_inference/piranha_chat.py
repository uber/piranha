import logging
import os
from pathlib import Path
from typing import List, Optional
from logger_formatter import CustomFormatter
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
keep capture groups in the "replace" string if they are meaningful (such as variable names, method names, etc.).

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
       
========================= Output Format =========================

<file_name_start> your_rule_name.toml <file_name_end>
```toml
# Define your rule within this section
[[rules]]
# Provide a unique name for your rule
name = "your_rule_name"

# Write a Tree-sitter query to identify the code pattern for refactoring. The outer most node should always be captured.
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

========================= Mistakes to avoid =========================

=== Infinite loops ===

Piranha rules are applied until fixedpoint. Meaning one should be very careful when substituting nodes to make sure 
the query stops matching once the cleanup is done. This can be achieved by using the "not_contains" filter or with (
#eq? ...) expressions in the query.

Example 1:

replace_node = "program"
replace = "X @program" 

... is generally a bad idea, because the query will keep matching the same node over and over again. To avoid you 
 NEED to specify a filter or constraint the query. That will stop the rule from matching the same node again.

Example 2:
query = """
(
    (class_declaration
        name: (identifier) @class_name
    ) @class_declaration
)
"""
replace_node = "class_name"
replace = "B"

... is also a bad idea. Since the query does not constraint the class name to be "A", the rule will match any class name.
Moreover, it will run into an infinite loop, because the rule will keep matching the same node over and over again.

=== Overly long matches in imports===

Sometimes queries can be significantly simplified by matching large subtrees. Making small queries is generally preferable
to matching a large subtree. 

Example 1:

query = """
(
    (import_declaration
        (scoped_identifier
            scope: (scoped_identifier
                scope: (scoped_identifier
                    scope: (scoped_identifier
                        scope: (scoped_identifier
                            scope: (scoped_identifier
                                scope: (identifier) @scope1
                                name: (identifier) @name1)
                            name: (identifier) @name2)
                        name: (identifier) @name3)
                    name: (identifier) @name4)
                name: (identifier) @name5)
            name: (identifier) @name6)
    ) @import_decl
    (#eq? @scope1 "com")
    (#eq? @name1 "uber")
    (#eq? @name2 "common")
    (#eq? @name3 "context")
    (#eq? @name4 "concurrent")
    (#eq? @name5 "MoreContextExecutors")
    (#eq? @name6 "directExecutor")
)
"""
... is bad! It can be simplified to:

query = """(
    (import_declaration 
        (_) @name) @import_decl
    (#eq? @name "com.uber.common.context.concurrent.MoreContextExecutors.directExecutor"))
"""

Always use the simplest query possible.

=== Not considering unnamed nodes ===

Tree-sitter produces parse trees rather than abstract syntax trees. This means that nodes in tree sitter
can contain children corresponding to punctuation, whitespace, and other tokens. This should be taken into account
when constructing the replacement string.

Example 1:

query = """(
    (method_invocation name: (_) @name
                       arguments: (argument_list) @args) @invk
    (#eq? @name @hole1))
"""

replace_node = "invk"
replace = "X.other_string(@args)"
... is wrong, because @args already contains the parentheses.

=== Forgetting parenthesis ===

If queries are not surrounded by parenthesis, tree-sitter will interpreter them as independent queries.

Example 1:

query = """
    (method_invocation name: (identifier) @name
                       arguments: (argument_list) @args) @invk
    (#eq? @name "directExecutor")
"""
... is a bad rule because the query is not surrounded by parenthesis.

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

=== Rules to improve === 

{rules}

=== Additional requirements === 

{hints}
========================= Please simplify the rules =========================

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
        default="gpt-4",
        validator=attr.validators.in_(["gpt-4", "gpt-4-32k", "gpt-3.5-turbo-16k"]),
    )

    def __attrs_post_init__(self):
        examples = self._get_examples("../../src/cleanup_rules/java")

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
                if file.endswith("rules.toml"):
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
