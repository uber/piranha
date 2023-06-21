import os
import time

import attr
from flask import Flask, request, jsonify, session
import openai
from flask import Flask, render_template
import logging

from rule_application import CodebaseRefactorer
from piranha_agent import PiranhaAgent
from flask_socketio import SocketIO, join_room

logger = logging.getLogger("Flask")
logger.setLevel(logging.DEBUG)
app = Flask(__name__)
socketio = SocketIO(app)


# Define data validation classes
@attr.s
class InferData:
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(["python", "java"]))
    hints = attr.ib(validator=attr.validators.instance_of(str))


@attr.s
class ImproveData:
    language = attr.ib(validator=attr.validators.in_(["python", "java"]))
    requirements = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=attr.validators.instance_of(str))


@attr.s
class RefactorData:
    language = attr.ib(validator=attr.validators.in_(["python", "java"]))
    folder_path = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=attr.validators.instance_of(str))


@app.route("/")
def home():
    return render_template("index.html")


@socketio.on("refactor_codebase")
def process_folder(data):
    data = RefactorData(**data)
    folder_path = data.folder_path

    refactorer = CodebaseRefactorer(data.language, data.folder_path, data.rules)
    refactorer.refactor_codebase(False)


@socketio.on("infer_piranha")
def infer_from_example(data):
    # Validate the data
    data = InferData(**data)
    openai.api_key = os.getenv("OPENAI_API_KEY")
    agent = PiranhaAgent(
        data.source_code,
        data.target_code,
        language=data.language,
        hints=data.hints,
    )

    room = session.get("room")
    join_room(room)

    rule_name, rule = agent.infer_rules(
        lambda intermediate_result: socketio.emit(
            "infer_progress",
            {"rule": intermediate_result},
            room=room,
        )
    )
    socketio.emit("infer_result", {"rule_name": rule_name, "rule": rule}, room=room)
    session["last_inference_result"] = {
        "rule_name": rule_name,
        "rule": rule,
    }


# New event listener
@socketio.on("improve_piranha")
def improve_rules(data):
    data = ImproveData(**data)

    room = session.get("room")
    join_room(room)

    time.sleep(1)

    socketio.emit("infer_result", {"rule_name": "", "rule": data.rules}, room=room)


if __name__ == "__main__":
    app.run(debug=True)
