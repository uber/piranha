import json
from typing import Any, Dict

import attr
from experimental.rule_inference.piranha_chat import PiranhaGPTChat

# Define the option constants
ANSWER_OPTIONS = ["yes", "no"]
IMPROVEMENT_OPTIONS = ["add filter", "change query", "change replacement", "give up"]
JSON_FILE_FORMAT = '{{"reasoning": "<your reasoning>", "answer": "{options}"}}'


class ControllerError(Exception):
    """Custom Exception class for handling Controller specific errors."""


@attr.s
class Controller:
    """A controller that decides what the AI model should do next, given the user request.

    :param PiranhaGPTChat chat: an instance of the PiranhaGPTChat model.
    """

    chat = attr.ib(type=PiranhaGPTChat)

    def get_model_selection(self, task_description: str, options: list) -> str:
        """Send the task description to the chat model and get the course of action the model has chosen.

        :param str task_description: Description of the task to be sent to the model.
        :param list options: List of valid options for the user's response.
        :return: The user's response as returned by the model.
        :rtype: str
        """
        self.chat.append_user_followup(task_description)
        n_tries = 3

        for _ in range(n_tries):
            try:
                completion = self.chat.get_model_response()
                completion = json.loads(completion)
                answer = completion.get("answer")
                if answer not in options:
                    raise ControllerError(
                        f"Invalid answer: {answer}. Expected one of: {', '.join(options)}."
                    )
                return answer
            except json.JSONDecodeError as e:
                self.chat.append_user_followup(
                    f"Error: {e}. Please respond in format: {JSON_FILE_FORMAT}\n"
                    f"Valid 'answer' options: {options}"
                )
        raise ControllerError(f"Failed to get valid answer after {n_tries} tries.")

    def should_improve_rule(self, task: str, rule: str) -> str:
        """Determines if a rule should be improved.

        :param str task: Task requested by the user.
        :param str rule: Rule to be checked for improvements.
        :return: The user's response as returned by the model.
        :rtype: str
        """
        task_description = (
            f"User requests improvements for: '{task}'.\n"
            f"Does the rule below need changes?\n\n{rule}\n"
            f"Response format: {JSON_FILE_FORMAT.format(options='yes/no')}. "
        )

        return self.get_model_selection(task_description, ANSWER_OPTIONS)

    def get_option_for_improvement(self, rule: str) -> str:
        """Asks the model to choose an improvement option for the given rule.

        :param str rule: Rule to be improved.
        :return: The user's response as returned by the model.
        :rtype: str
        """
        task_description = (
            f"You opted to improve this rule:\n{rule}\n"
            f"Choose an improvement option and state your reasoning and selection. "
            f"Response format: {JSON_FILE_FORMAT.format(options='/'.join(IMPROVEMENT_OPTIONS))}."
        )

        return self.get_model_selection(task_description, IMPROVEMENT_OPTIONS)
