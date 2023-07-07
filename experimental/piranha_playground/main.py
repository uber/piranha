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
import os
import sys

import openai
from piranha_playground.data_validation import (
    ImproveData,
    InferData,
    RefactorData,
    RefactorSnippet,
)
from piranha_playground.rule_inference.piranha_agent import (
    PiranhaAgent,
    PiranhaAgentError,
)
from piranha_playground.rule_inference.rule_application import (
    CodebaseRefactorer,
    CodebaseRefactorerException,
)
from flask import Flask, render_template, request
from flask_socketio import SocketIO

from piranha_playground.rule_inference.utils.logger_formatter import CustomFormatter

# Create Flask app and SocketIO app
app = Flask(__name__)
socketio = SocketIO(app, ping_timeout=300, ping_interval=5)

# Initialize session dict
socketio_sessions = {}

# Configure logging

logger = logging.getLogger("FlaskApp")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
ch.setLevel(logging.INFO)
ch.setFormatter(CustomFormatter())
logger.addHandler(ch)


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

    :param data: A dictionary containing the necessary information to perform the refactoring.
    """
    try:
        data = RefactorData(**data)
        refactorer = CodebaseRefactorer(data.language, data.folder_path, data.rules)
        refactorer.refactor_codebase(False)
        socketio.emit("refactor_result", {"result": True})
    except (ValueError, CodebaseRefactorerException) as e:
        socketio.emit("refactor_result", {"result": False})


@socketio.on("infer_piranha")
def infer_from_example(data):
    """
    Event handler for the infer_piranha event.
    Infers coding rules based on source and target code examples.

    :param data: A dictionary containing the source and target code examples and the programming language.
    """
    try:
        data = InferData(**data)
        agent = PiranhaAgent(
            data.source_code,
            data.target_code,
            language=data.language,
            hints="",
        )
        static_rules = agent.infer_rules_statically()
        socketio.emit("infer_progress", {"rule": static_rules})
        rules = agent.infer_rules()
        socketio_sessions[request.sid]["agent"] = agent
        socketio.emit(
            "infer_success",
            {
                "rule": rules,
                "gpt_output": agent.get_explanation(),
            },
        )
    except (ValueError, PiranhaAgentError) as e:
        socketio.emit("infer_error", {"error": str(e)})


@socketio.on("improve_piranha")
def improve_rules(data):
    """
    Event handler for the improve_piranha event.
    Improves the inferred coding rules.

    :param data: A dictionary containing the requirements and current rules.
    """
    try:
        data = ImproveData(**data)
        agent: PiranhaAgent = socketio_sessions[request.sid].get("agent")
        rules = agent.improve_rule(data.requirements, data.rules)
        socketio.emit(
            "infer_result",
            {
                "rule": rules,
                "gpt_output": agent.get_explanation(),
            },
        )
    except (ValueError, PiranhaAgentError, AttributeError) as e:
        socketio.emit("infer_error", {"error": str(e)})


@socketio.on("test_rule")
def test_rule(data):
    """
    Event handler for the test_rule event.
    Tests the inferred rules by applying them to the provided source code.

    :param data: A dictionary containing the language, rules, and source code.
    """
    try:
        data = RefactorSnippet(**data)
        refactored_code = CodebaseRefactorer.refactor_snippet(
            data.source_code, data.language, data.rules
        )
        socketio.emit("test_result", {"refactored_code": refactored_code})
    except (ValueError, CodebaseRefactorerException) as e:
        socketio.emit("test_error", {"error": str(e)})


def main():
    openai.api_key = os.getenv("OPENAI_API_KEY")
    if not openai.api_key:
        sys.exit(
            "Please set the OPENAI_API_KEY environment variable to your OpenAI API key."
        )
    logger.info(f"Starting server. Listening at: http://127.0.0.1:5000")
    socketio.run(app)


if __name__ == "__main__":
    main()
