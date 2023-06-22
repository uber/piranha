import attr
import toml
from typing import List
from polyglot_piranha import (
    Rule,
    PiranhaArguments,
    RuleGraph,
    execute_piranha,
    PiranhaOutputSummary,
)


@attr.s
class CodebaseRefactorer:
    """
    A class that uses Piranha to refactor an entire codebase based on rules specified in a .toml file.
    """

    language = attr.ib(type=str)
    path_to_codebase = attr.ib(type=str)
    rules = attr.ib(type=str)
    include_paths = attr.ib(type=List[str], default=None)
    exclude_paths = attr.ib(type=List[str], default=None)

    def refactor_codebase(self, dry_run: bool = True) -> List[PiranhaOutputSummary]:
        """
        Applies the refactoring rules to the codebase.
        Returns a list of piranha summaries
        """
        # Load the rules from the .toml file
        toml_dict = toml.loads(self.rules)

        rules = toml_dict.get("rules", [])
        if not rules:
            raise Exception("TOML does not include any rule specifications.")

        # Create the Piranha rule graph
        rule_graph = self.create_rule_graph(rules)

        # Create the PiranhaArguments object
        args = PiranhaArguments(
            language=self.language,
            path_to_codebase=self.path_to_codebase,
            rule_graph=rule_graph,
            dry_run=dry_run,
        )

        # Execute the refactoring
        return execute_piranha(args)

    @staticmethod
    def create_rule_graph(toml_rules: List[dict]) -> RuleGraph:
        """
        Creates a Piranha RuleGraph object based on a list of rules from a .toml file.

        :param toml_rules: list, The list of rules from the .toml file.
        :return: RuleGraph, The created RuleGraph object.
        """
        rules = []
        for toml_rule in toml_rules:
            rule = Rule(
                name=toml_rule["name"],
                query=toml_rule["query"],
                replace_node=toml_rule["replace_node"],
                replace=toml_rule["replace"],
            )
            rules.append(rule)

        return RuleGraph(rules=rules, edges=[])
