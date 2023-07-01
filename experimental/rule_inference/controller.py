import json
from typing import Any, Dict

import attr
import toml

from experimental.rule_inference.piranha_chat import PiranhaGPTChat
from experimental.rule_inference.utils.pretty_toml import PrettyTOML

# Define the option constants
ANSWER_OPTIONS = ["yes", "no"]
IMPROVEMENT_OPTIONS = ["add filter", "change query", "change replacement", "give up"]
JSON_FILE_FORMAT = '{{"reasoning": "<your reasoning>", "answer": "{options}"}}'


class ControllerError(Exception):
    """Custom Exception class for handling Controller specific errors."""


@attr.s
class Controller:
    """A class used to represent the Controller which takes user input and sends it to the model,
    then handles the model's response.

    :param PiranhaGPTChat chat: an instance of the PiranhaGPTChat model.
    """

    chat: PiranhaGPTChat = attr.ib()

    def get_user_answer(self, task_description: str, options: list) -> str:
        """Send the task description to the chat model and parse the model's response.

        :param str task_description: Description of the task to be sent to the model.
        :param list options: List of valid options for the user's response.
        :return: The user's response as returned by the model.
        :rtype: str
        """
        self.chat.append_user_followup(task_description)

        while True:
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

        return self.get_user_answer(task_description, ANSWER_OPTIONS)

    def get_option_for_improvement(self, rule: str) -> str:
        """Requests from the user an option for improvement of a rule.

        :param str rule: Rule to be improved.
        :return: The user's response as returned by the model.
        :rtype: str
        """
        task_description = (
            f"You opted to improve this rule:\n{rule}\n"
            f"Choose an improvement option and state your reasoning and selection. "
            f"Response format: {JSON_FILE_FORMAT.format(options='/'.join(IMPROVEMENT_OPTIONS))}."
        )

        return self.get_user_answer(task_description, IMPROVEMENT_OPTIONS)
