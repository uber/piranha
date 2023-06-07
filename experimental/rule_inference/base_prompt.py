import os
from pathlib import Path


class BasePrompt:
    explanation = '''
    Your task is to create refactoring rules for Polyglot Piranha, a tool that uses tree-sitter for parsing and refactoring code.
    Each rule will transform an original code snippet into a provided refactored version.
    As input you will receive the original and refactored snippets and their tree-sitter representations.
    
    The rule should be in Polyglot Piranha's domain-specific language (DSL). Explanations and examples of the DSL are below.
    
    Your rule should accurately capture the transformation from the original to the refactored code.
    It should be specific enough to avoid matching unrelated code patterns, but general enough to include code from captured groups where possible.
    
    ========================= DSL Explanation =========================
    
    The TOML file should contain at least one rule with the following properties:
    
    - "query": Tree-sitter query to find the code pattern to refactor
    - "replace_node": The captured node in the query that will be replaced
    - "replace_string": Replacement string or pattern for the refactored code
    - "holes": Placeholders in your queries that will be instantiated at runtime
    
    Additionally, the rule can have optional properties such as "is_seed_rule", "groups", and "filters".
    Filters can have properties like "enclosing_node", "not_contains", "contains", "at_least", "at_most".
    The filters are used to specify conditions for the rule to be applied.
    
    ========================= Output Format =========================
    
    <file_name_start> your_rules_file.toml <file_name_end>
    ```toml
    # Define your rule within this section
    [[rules]]
    # Provide a unique name for your rule
    name = "your_rule_name"
    
    # Write a Tree-sitter query to identify the code pattern for refactoring
    query = """(
        (method_invocation name: (_) @name
                           arguments: (_) @args) @invk
        (#eq? @name "some_string"))
    """
    
    # Specify the captured node from the query that will be replaced
    replace_node = "invk"
    
    # Replacement string that will substitute `replace_node`
    replace = "X.other_string(@args)"
    
    # Specify any placeholders in your queries that will be filled in at runtime
    holes = ["hole1", "hole2"]
    
    # Specify if this rule should be triggered first. If it depends on other rules, set to false
    is_seed_rule = true
    
    # If necessary, define filters for your rule
    [[rules.filters]]
    # This pattern should match any ancestor of the captured node (optional)
    enclosing_node = "(your_enclosing_node_pattern) @your_capture_name"
    
    # Define patterns that should not be present within the enclosing_node (optional)
    not_contains = [
        """
        (your_not_contains_pattern) @your_capture_name
        """
    ]
    # Define a pattern that should be present within the enclosing_node (optional)
    contains =
        """
        (your_contains_pattern) @your_capture_name
        """
    # Define the minimum and maximum number of children that should match the 'contains' pattern (optional)
    at_least = 1
    at_most = 5
    ```
    
    ========================= Rule Examples =========================
    '''
    input_template = """
    ========================= Task =========================

    ### Original Code ###
    
    {source_code}
    
    ### Refactored Code ###
    
    {target_code}
    
    ### Diff ###
    
    {diff}
    
    ### Tree-sitter Representation of Original Code ###
    
    {source_tree}
    
    ### Tree-sitter Representation of Refactored Code ###
    
    {target_tree}
    
    ========================= Output =========================
    """

    @staticmethod
    def generate_prompt(**kwargs):
        examples = BasePrompt._get_examples("../../src/cleanup_rules")
        formatted = (
            BasePrompt.explanation
            + "\n"
            + examples
            + "\n"
            + BasePrompt.input_template.format(**kwargs)
        )
        return [{"role": "user", "content": formatted}]

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
