import os

import attr
from flask import Flask, request, jsonify, session
import openai
from flask import Flask, render_template
import logging
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
class FolderData:
    folder_path = attr.ib(validator=attr.validators.instance_of(str))


@app.route("/")
def home():
    return render_template("index.html")


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

    return jsonify({"message": f"Received source code: {data.source_code}"}), 200


@app.route("/api/process_folder", methods=["POST"])
def process_folder():
    data = request.get_json()
    data = FolderData(**data)
    folder_path = data.folder_path

    # Use the folder_path variable to process the folder.
    # Note: This assumes your server has the appropriate permissions to access and read the directory.

    # Let's just return a message for this example
    return jsonify({"message": f"Received folder path: {folder_path}"}), 200


if __name__ == "__main__":
    app.run(debug=True)
