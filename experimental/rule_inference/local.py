import logging
import multiprocessing
import os
import sys
import time

import attr
import openai
import toml
from flask import Flask, render_template, session
from flask_socketio import SocketIO, join_room
from utils.pretty_toml import PrettyTOML

from experimental.rule_inference.piranha_agent import (
    PiranhaAgent,
    run_piranha_with_timeout,
)
from experimental.rule_inference.rule_application import CodebaseRefactorer
from experimental.rule_inference.utils.rule_utils import RawRuleGraph

# Configure logging
logger = logging.getLogger("Flask")

# Create Flask app and SocketIO app
app = Flask(__name__)
socketio = SocketIO(app, ping_timeout=300, ping_interval=5)

logging.getLogger("werkzeug").setLevel(logging.INFO)


def valid_toml(instance, attribute, value):
    try:
        toml.loads(value)
    except toml.TomlDecodeError as e:
        raise ValueError("Invalid TOML data") from e


@attr.s
class InferData:
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))


@attr.s
class ImproveData:
    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))
    requirements = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=valid_toml)

    def __attrs_post_init__(self):
        self.rules = toml.dumps(toml.loads(self.rules), encoder=PrettyTOML())


@attr.s
class RefactorData:
    language = attr.ib(validator=attr.validators.in_(["kt", "java"]))
    folder_path = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=attr.validators.instance_of(str))


@app.route("/")
def home():
    return render_template("index.html")


@socketio.on("refactor_codebase")
def process_folder(data):
    data = RefactorData(**data)
    refactorer = CodebaseRefactorer(data.language, data.folder_path, data.rules)
    success, summaries = refactorer.refactor_codebase(False)
    if success:
        socketio.emit("refactor_progress", {"result": "Success"})
    else:
        socketio.emit("refactor_progress", {"result": "Failed"})


@socketio.on("infer_piranha")
def infer_from_example(data):
    # Validate the data
    data = InferData(**data)
    agent = PiranhaAgent(
        data.source_code,
        data.target_code,
        language=data.language,
        hints="",
    )

    room = session.get("room")
    join_room(room)

    first_it = agent.infer_rules_init()
    socketio.emit(
        "infer_progress",
        {"rule": first_it, "gpt_output": ""},
        room=room,
    )

    rule_name, rule = agent.infer_rules()

    session["agent"] = agent
    socketio.emit(
        "infer_result",
        {
            "rule_name": rule_name,
            "rule": rule,
            "gpt_output": agent.get_explanation(),
        },
        room=room,
    )


@socketio.on("improve_piranha")
def improve_rules(data):
    data = ImproveData(**data)
    room = session.get("room")
    join_room(room)

    agent: PiranhaAgent = session.get("agent")
    rule_name, rule = agent.improve_rule(data.requirements, data.rules)
    socketio.emit(
        "infer_result",
        {
            "rule_name": rule_name,
            "rule": rule,
            "gpt_output": agent.get_explanation(),
        },
        room=room,
    )


@socketio.on("test_rule")
def test_rule(data):
    # Extract data
    language = data.get("language", "")
    rules = data.get("rules", "")
    source_code = data.get("source_code", "")

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
        test_result = "Success" if success else "Error"

    except multiprocessing.context.TimeoutError as e:
        test_result = "Error"
        refactored_code = "Piranha is in an infinite loop."
    except Exception as e:
        test_result = "Error"
        refactored_code = str(e)

    # Emit the result
    socketio.emit(
        "test_result",
        {
            "test_result": test_result,
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
