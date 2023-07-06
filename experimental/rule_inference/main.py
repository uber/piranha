import logging
import os
import sys

import openai
from experimental.rule_inference.data_validation import (ImproveData,
                                                         InferData,
                                                         RefactorData,
                                                         RefactorSnippet)
from experimental.rule_inference.piranha_agent import PiranhaAgent
from experimental.rule_inference.rule_application import CodebaseRefactorer
from flask import Flask, render_template, request
from flask_socketio import SocketIO

# Create Flask app and SocketIO app
app = Flask(__name__)
socketio = SocketIO(app, ping_timeout=300, ping_interval=5)

# Initialize session dict
socketio_sessions = {}

# Configure logging
logging.getLogger("werkzeug").setLevel(logging.ERROR)


@app.route("/")
def home():
    """
    The main route that returns the index.html template.
    """
    return render_template("index.html")


@socketio.on("connect")
def on_connect():
    """
    Event handler for new client connections.
    Initializes session data for the client.
    """
    socketio_sessions[request.sid] = {}


@socketio.on("disconnect")
def on_disconnect():
    """
    Event handler for client disconnections.
    Removes session data for the client.
    """
    del socketio_sessions[request.sid]


@socketio.on("refactor_codebase")
def process_folder(data):
    """
    Event handler for the refactor_codebase event.
    Attempts to refactor a codebase based on the provided rules.
    Emits the refactor_progress event with the result of the refactoring attempt.

    :param data: A dictionary containing the necessary information to perform the refactoring.
    """
    data = RefactorData(**data)
    refactorer = CodebaseRefactorer(data.language, data.folder_path, data.rules)
    success, _ = refactorer.refactor_codebase(False)
    socketio.emit("refactor_result", {"result": success})


@socketio.on("infer_piranha")
def infer_from_example(data):
    """
    Event handler for the infer_piranha event.
    Infers coding rules based on source and target code examples.

    :param data: A dictionary containing the source and target code examples and the programming language.
    """
    data = InferData(**data)
    agent = PiranhaAgent(
        data.source_code,
        data.target_code,
        language=data.language,
        hints="",
    )

    static_rules = agent.infer_rules_statically()
    socketio.emit("infer_progress", {"rule": static_rules})

    # Store the agent in case the user wants to improve the rule
    result, rule = agent.infer_rules()
    socketio_sessions[request.sid]["agent"] = agent
    socketio.emit(
        "infer_result",
        {
            "result": result,
            "rule": rule,
            "gpt_output": agent.get_explanation(),
        },
    )


@socketio.on("improve_piranha")
def improve_rules(data):
    """
    Event handler for the improve_piranha event.
    Improves the inferred coding rules.

    :param data: A dictionary containing the requirements and current rules.
    """
    data = ImproveData(**data)
    agent: PiranhaAgent = socketio_sessions[request.sid].get("agent")
    success, rule = agent.improve_rule(data.requirements, data.rules)
    socketio.emit(
        "infer_result",
        {
            "result": success,
            "rule": rule,
            "gpt_output": agent.get_explanation(),
        },
    )


@socketio.on("test_rule")
def test_rule(data):
    """
    Event handler for the test_rule event.
    Tests the inferred rules by applying them to the provided source code.

    :param data: A dictionary containing the language, rules, and source code.
    """
    data = RefactorSnippet(**data)
    success, refactored_code = CodebaseRefactorer.refactor_snippet(
        data.source_code, data.language, data.rules
    )
    socketio.emit(
        "test_result",
        {
            "result": success,
            "refactored_code": refactored_code,
        },
    )


if __name__ == "__main__":
    openai.api_key = os.getenv("OPENAI_API_KEY")
    if not openai.api_key:
        sys.exit(
            "Please set the OPENAI_API_KEY environment variable to your OpenAI API key."
        )
    app.run(debug=True)
