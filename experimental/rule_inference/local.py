import os

from flask import Flask, request, jsonify
import openai
from flask import Flask, render_template
import logging
from piranha_agent import PiranhaAgent


logging.getLogger("Flask").setLevel(logging.DEBUG)
app = Flask(__name__)


@app.route("/")
def home():
    return render_template("index.html")


@app.route("/api/infer_piranha", methods=["POST"])
def infer_from_example():
    data = request.get_json()
    openai.api_key = os.getenv("OPENAI_API_KEY")
    agent = PiranhaAgent(
        data["source_code"],
        data["target_code"],
        language=data["language"],
        hints=data["hints"],
    )

    rule_name, rule = agent.infer_rules()
    return jsonify(rule_name, rule), 200


if __name__ == "__main__":
    app.run(debug=True)
