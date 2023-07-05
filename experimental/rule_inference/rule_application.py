import logging
import multiprocessing
from typing import List, Optional, Tuple

import attr
import toml
from polyglot_piranha import (PiranhaArguments, PiranhaOutputSummary, Rule,
                              RuleGraph, execute_piranha)

from experimental.rule_inference.utils.logger_formatter import CustomFormatter
from experimental.rule_inference.utils.rule_utils import RawRuleGraph

logger = logging.getLogger("CodebaseRefactorer")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


def enable_piranha_logs():
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.DEBUG)


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

    def refactor_codebase(
        self, dry_run: bool = True
    ) -> Tuple[bool, List[PiranhaOutputSummary]]:
        """
        Applies the refactoring rules to the codebase.
        Returns a list of piranha summaries
        """
        # Load the rules from the .toml file

        try:
            toml_dict = toml.loads(self.rules)
            rule_graph = RawRuleGraph.from_toml(toml_dict)

            # Create the Piranha rule graph

            # Create the PiranhaArguments object
            args = PiranhaArguments(
                language=self.language,
                path_to_codebase=self.path_to_codebase,
                rule_graph=rule_graph.to_graph(),
                dry_run=dry_run,
                substitutions=toml_dict.get("substitutions", [{}])[0],
            )

            output_summaries = execute_piranha(args)
            logger.info("Changed files:")
            for summary in output_summaries:
                logger.info(summary.path)

            # Execute the refactoring
            return True, output_summaries
        except BaseException as e:
            logger.error(e)
            return False, []

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
