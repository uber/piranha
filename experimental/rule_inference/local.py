import os
import time
import logging
import attr
import openai

from flask import Flask, render_template, session
from flask_socketio import SocketIO, join_room

from rule_application import CodebaseRefactorer
from piranha_agent import PiranhaAgent

# Configure logging
logger = logging.getLogger("Flask")

# Create Flask app and SocketIO app
app = Flask(__name__)
socketio = SocketIO(app)


# Data validation classes
@attr.s
class InferData:
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(["python", "java"]))


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
    agent = PiranhaAgent(
        data.source_code,
        data.target_code,
        language=data.language,
        hints="",
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


# New event listener
@socketio.on("improve_piranha")
def improve_rules(data):
    data = ImproveData(**data)
    room = session.get("room")
    join_room(room)
    time.sleep(1)
    socketio.emit("infer_result", {"rule_name": "", "rule": data.rules}, room=room)


if __name__ == "__main__":
    openai.api_key = os.getenv("OPENAI_API_KEY")
    app.run(debug=True)
