import json
from unittest.mock import Mock

import pytest

from experimental.rule_inference.controller import Controller, ControllerError


def setup_controller(mock_response):
    # Create a mock chat model
    mock_chat = Mock()

    # Define the response of the get_model_response method
    mock_chat.get_model_response.return_value = mock_response

    # Initialize the controller with the mock chat
    controller = Controller(chat=mock_chat)

    return controller, mock_chat


def test_get_user_answer_correct():
    task_description = "Test task description"
    options = ["yes", "no"]
    correct_answer = "yes"
    valid_json_correct_answer = json.dumps(
        {"reasoning": "Test reasoning", "answer": correct_answer}
    )

    controller, mock_chat = setup_controller(valid_json_correct_answer)

    assert controller.get_user_answer(task_description, options) == correct_answer
    mock_chat.append_user_followup.assert_called_once_with(task_description)


def test_get_user_answer_incorrect():
    task_description = "Test task description"
    options = ["yes", "no"]
    incorrect_answer = "maybe"
    valid_json_incorrect_answer = json.dumps(
        {"reasoning": "Test reasoning", "answer": incorrect_answer}
    )

    controller, mock_chat = setup_controller(valid_json_incorrect_answer)

    with pytest.raises(ControllerError):
        controller.get_user_answer(task_description, options)
    mock_chat.append_user_followup.assert_called_once_with(task_description)


def test_get_user_answer_invalid_json():
    task_description = "Test task description"
    options = ["yes", "no"]
    correct_answer = "yes"
    invalid_json = "Invalid JSON"
    valid_json_correct_answer = json.dumps(
        {"reasoning": "Test reasoning", "answer": correct_answer}
    )

    controller, mock_chat = setup_controller(invalid_json)

    # After the invalid JSON response, the model responds with a correct answer
    mock_chat.get_model_response.side_effect = [invalid_json, valid_json_correct_answer]

    assert controller.get_user_answer(task_description, options) == correct_answer
    assert mock_chat.append_user_followup.call_count == 2


def test_should_improve_rule():
    task_description = "Test task description"
    rule = "Test rule"
    correct_answer = "yes"
    valid_json_correct_answer = json.dumps(
        {"reasoning": "Test reasoning", "answer": correct_answer}
    )

    controller, mock_chat = setup_controller(valid_json_correct_answer)

    assert controller.should_improve_rule(task_description, rule) == correct_answer
    mock_chat.append_user_followup.assert_called_once()


def test_get_option_for_improvement():
    rule = "Test rule"
    correct_answer = "add filter"
    valid_json_correct_answer = json.dumps(
        {"reasoning": "Test reasoning", "answer": correct_answer}
    )

    controller, mock_chat = setup_controller(valid_json_correct_answer)

    assert controller.get_option_for_improvement(rule) == correct_answer
    mock_chat.append_user_followup.assert_called_once()
