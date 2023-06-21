import os
import attr
from flask import Flask, request, jsonify, session, render_template
from flask_socketio import SocketIO, join_room
import openai
import logging
from piranha_agent import PiranhaAgent


# Define data validation classes
@attr.s
class InferData:
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(["Python", "Java"]))
    hints = attr.ib(validator=attr.validators.instance_of(str))


@attr.s
class FolderData:
    folder_path = attr.ib(validator=attr.validators.instance_of(str))


# Initialize Flask and SocketIO
logging.getLogger("Flask").setLevel(logging.DEBUG)
app = Flask(__name__)
socketio = SocketIO(app)
openai.api_key = os.getenv("OPENAI_API_KEY")  # Set API key at start


# Routes
@app.route("/")
def home():
    return render_template("index.html")


@socketio.on("infer_piranha")
def infer_from_example(data):
    try:
        data = InferData(**data)
    except (TypeError, ValueError) as e:
        return {"error": str(e)}

    agent = PiranhaAgent(
        data.source_code,
        data.target_code,
        language=data.language,
        hints=data.hints,
    )

    room = session.get("room")
    join_room(room)

    try:
        while True:  # continuously emit results
            rule_name, rule = agent.infer_rules()
            socketio.emit(
                "infer_result", {"rule_name": rule_name, "rule": rule}, room=room
            )
            socketio.sleep(1)  # Sleep for a second to prevent overloading the client
    except Exception as e:
        socketio.emit("infer_error", {"error": str(e)}, room=room)

    return {
        "message": "Inference started"
    }  # Acknowledge event reception and start of inference


@app.route("/api/process_folder", methods=["POST"])
def process_folder():
    try:
        data = FolderData(**request.get_json())
    except (TypeError, ValueError) as e:
        return jsonify({"error": str(e)}), 400

    folder_path = data.folder_path
    # TODO: Process the folder
    return jsonify({"message": f"Received folder path: {folder_path}"}), 200


if __name__ == "__main__":
    app.run(debug=True)
