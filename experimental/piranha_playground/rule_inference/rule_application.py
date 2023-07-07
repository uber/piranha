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
import multiprocessing
from typing import List, Optional, Tuple

import attr
import toml
from piranha_playground.rule_inference.utils.logger_formatter import CustomFormatter
from piranha_playground.rule_inference.utils.rule_utils import RawRuleGraph
from polyglot_piranha import (
    PiranhaArguments,
    PiranhaOutputSummary,
    Rule,
    RuleGraph,
    execute_piranha,
)

logger = logging.getLogger("CodebaseRefactorer")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.DEBUG)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


class CodebaseRefactorerException(Exception):
    """
    Exception class for CodebaseRefactorer.
    """

    pass


def enable_piranha_logs():
    """
    Sets up the logging configurations for Piranha.
    """
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.DEBUG)


def _run_piranha_with_timeout_aux(
    source_code: str, language: str, raw_graph: RawRuleGraph, substitutions: dict
):
    """
    Private method to run Piranha with a timeout. Executes Piranha with provided arguments.

    :param source_code: str: The source code to be refactored.
    :param language: str: The language of the source code.
    :param raw_graph: RawRuleGraph: The rule graph to be used for refactoring.
    :param substitutions: dict: The substitutions to be made during refactoring.

    :return: Tuple[str, bool]: The refactored code and a boolean indicating if the execution was successful.
    """
    try:
        # Prepare arguments for Piranha execution
        args = PiranhaArguments(
            code_snippet=source_code,
            language=language,
            rule_graph=raw_graph.to_graph(),
            dry_run=True,
            substitutions=substitutions,
        )
        piranha_results = execute_piranha(args)
        # Check if the execution returns results, if yes then return the content of the first result
        # Otherwise, return an empty list
        if piranha_results:
            return piranha_results[0].content, True
        return source_code, True
    except BaseException as e:
        return str(e), False


def run_piranha_with_timeout(
    source_code: str,
    language: str,
    raw_graph: RawRuleGraph,
    substitutions: Optional[dict] = None,
    timeout: Optional[int] = 10,
) -> Tuple[str, bool]:
    """
    Executes Piranha with a timeout. Calls a private method to perform the execution and terminates if the timeout is reached.

    :param source_code: str: The source code to be refactored.
    :param language: str: The language of the source code.
    :param raw_graph: RawRuleGraph: The rule graph to be used for refactoring.
    :param substitutions: Optional[dict]: The substitutions to be made during refactoring. Default is None.
    :param timeout: Optional[int]: The timeout for the execution in seconds. Default is 10 seconds.

    :return: Tuple[str, bool]: The refactored code and a boolean indicating if the execution was successful.
    """
    with multiprocessing.Pool(processes=1) as pool:
        async_result = pool.apply_async(
            _run_piranha_with_timeout_aux,
            (source_code, language, raw_graph, substitutions),
        )
        return async_result.get(timeout=timeout)


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

        :param dry_run: bool: A boolean that if true, runs the refactor without making actual changes. Default is True.

        :return: List[PiranhaOutputSummary]: A list of summaries of the changes made by Piranha.
        :raises CodebaseRefactorerException: If the refactoring fails.
        """
        try:
            toml_dict = toml.loads(self.rules)
            rule_graph = RawRuleGraph.from_toml(toml_dict)
            args = PiranhaArguments(
                language=self.language,
                path_to_codebase=self.path_to_codebase,
                rule_graph=rule_graph.to_graph(),
                dry_run=dry_run,
                substitutions=toml_dict.get("substitutions", [{}])[0],
                allow_dirty_ast=True,
            )

            output_summaries = execute_piranha(args)
            logger.info("Changed files:")
            for summary in output_summaries:
                logger.info(summary.path)
            return output_summaries
        except BaseException as e:
            raise CodebaseRefactorerException(str(e)) from e

    @staticmethod
    def refactor_snippet(source_code: str, language: str, rules: str) -> str:
        """
        Refactors a code snippet based on the provided rules.

        :param source_code: str: The source code to be refactored.
        :param language: str: The language of the source code.
        :param rules: str: The refactoring rules in a .toml format.

        :return: str: The refactored code or error message.
        :raises CodebaseRefactorerException: If the refactoring fails.
        """
        try:
            toml_dict = toml.loads(rules)
            substitutions = toml_dict.get("substitutions", [{}])[0]
            refactored_code, success = run_piranha_with_timeout(
                source_code,
                language,
                RawRuleGraph.from_toml(toml_dict),
                timeout=5,
                substitutions=substitutions,
            )
            return refactored_code
        except multiprocessing.context.TimeoutError as e:
            raise CodebaseRefactorerException(
                "Piranha is likely in an infinite loop. Please check your rules."
            ) from e
        except Exception as e:
            raise CodebaseRefactorerException(str(e)) from e
