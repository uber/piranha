#  Copyright (c) 2023 Uber Technologies, Inc.

#  <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
#  except in compliance with the License. You may obtain a copy of the License at
#  <p>http://www.apache.org/licenses/LICENSE-2.0

#  <p>Unless required by applicable law or agreed to in writing, software distributed under the
#  License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
#  express or implied. See the License for the specific language governing permissions and
#  limitations under the License.

from abc import ABC, abstractmethod
from typing import Any, Union, List, Dict
from polyglot_piranha import PiranhaArguments, execute_piranha, Rule, RuleGraph, OutgoingEdges


class ExecutePiranha(ABC):
    '''
    This abstract class implements the higher level strategy for
    applying a specific polyglot piranha configuration i.e. rules/edges.
    '''
    
    def __init__(self, paths_to_codebase: List[str], language: str, substitutions: Dict[str, str], dry_run=False, allow_dirty_ast=False):
        self.paths_to_codebase = paths_to_codebase
        self.language = language
        self.substitutions = substitutions
        self.dry_run = dry_run
        self.allow_dirty_ast = allow_dirty_ast
    
    def __call__(self) -> dict:
        piranha_args = self.get_piranha_arguments()
        self.summaries = execute_piranha(piranha_args)

        output = self.summaries_to_custom_dict(self.summaries)
        success = True

        if not output:
            success = False
            output = {}
        output[self.step_name()] = success
        return output

    @abstractmethod
    def step_name(self) -> str:
        '''
        The overriding method should return the name of the strategy.
        '''
        ...

    @abstractmethod
    def get_rules(self) -> List[Rule]:
        '''
        The list of rules.
        '''
        ...

    def get_edges(self) -> List[OutgoingEdges]:
        '''
        The list of edges.
        '''
        return []

    def get_rule_graph(self) -> RuleGraph:
        '''
        Strategy to construct a rule graph from rules/edges.
        '''
        return RuleGraph(rules=self.get_rules(), edges=self.get_edges())

    def path_to_configuration(self) -> Union[None, str]:
        '''
        Path to rules/edges toml file (incase rule graph is not specified).
        '''
        return None

    def get_piranha_arguments(self) -> PiranhaArguments:
        rg = self.get_rule_graph()
        path_to_conf = self.path_to_configuration()
        if rg.rules and path_to_conf:
            raise Exception(
                "You have provided a rule graph and path to configurations. Do not provide both.")
        if not rg.rules and not path_to_conf:
            raise Exception("You have neither provided a rule graph nor path to configurations.")
        if rg.rules:
            return PiranhaArguments(
                language=self.language,
                paths_to_codebase=self.paths_to_codebase,
                substitutions=self.substitutions,
                rule_graph=self.get_rule_graph(),
                cleanup_comments=True,
                dry_run=self.dry_run,
                allow_dirty_ast=self.allow_dirty_ast
            )
        return PiranhaArguments(
            language=self.language,
            paths_to_codebase=self.paths_to_codebase,
            substitutions=self.substitutions,
            path_to_configurations=self.path_to_configuration(),
            cleanup_comments=True,
            dry_run=self.dry_run,
            allow_dirty_ast=self.allow_dirty_ast
        )

    def get_matches(self, specified_rule: str) -> List[dict]:
        """
        This function gets matches for a specified rule.
        """
        return [match.matches
                for summary in self.summaries
                for actual_rule, match in summary.matches if specified_rule == actual_rule]

    @abstractmethod
    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        '''
        The overriding method should implement the logic for extracting out the
        useful information from the matches/rewrites reported by polyglot piranha into a dict.
        '''
        ...
