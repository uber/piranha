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
from flask import Flask, Response, jsonify, render_template, request, session

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
from piranha_playground.rule_inference.utils.logger_formatter import CustomFormatter

# Create Flask app
app = Flask(__name__)

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


@app.route("/refactor_codebase", methods=["POST"])
def process_folder():
    """
    Route for the refactor_codebase event.
    Attempts to refactor a codebase based on the provided rules.

    :param data: A dictionary containing the necessary information to perform the refactoring.
    """
    data = request.get_json()
    try:
        data = RefactorData(**data)
        refactorer = CodebaseRefactorer(data.language, data.folder_path, data.rules)
        refactorer.refactor_codebase(False)
        return jsonify({"result": True})
    except (ValueError, CodebaseRefactorerException) as e:
        return jsonify({"result": False, "error": str(e)}), 400


@app.route("/infer_rule_graph", methods=["POST"])
def infer_static_rule():
    """
    Route for the infer_static_rule event.
    Infers static coding rules based on source and target code examples.

    :param data: A dictionary containing the source and target code examples and the programming language.
    """
    data = request.get_json()
    try:
        data = InferData(**data)
        agent = PiranhaAgent(
            data.source_code,
            data.target_code,
            language=data.language,
            hints="",
        )
        static_rules = agent.infer_rules_statically()
        return jsonify({"rules": static_rules})
    except (ValueError, PiranhaAgentError) as e:
        return jsonify({"error": str(e)}), 400


@app.route("/improve_rule_graph", methods=["POST"])
def improve_rules():
    """
    Route for the improve_piranha event.
    Improves the inferred coding rules.

    :param data: A dictionary containing the requirements and current rules.
    """
    data = request.get_json()
    try:
        data = ImproveData(**data)
        agent: PiranhaAgent = PiranhaAgent(
            data.source_code,
            data.target_code,
            language=data.language,
            hints="",
        )
        rules = agent.improve_rule_graph(data.requirements, data.rules, data.option)
        return jsonify(
            {
                "rule": rules,
                "gpt_output": agent.get_explanation(),
            }
        )
    except (ValueError, PiranhaAgentError, AttributeError) as e:
        return jsonify({"error": str(e)}), 400


@app.route("/test_rule", methods=["POST"])
def test_rule():
    """
    Route for the test_rule event.
    Tests the inferred rules by applying them to the provided source code.

    :param data: A dictionary containing the language, rules, and source code.
    """
    data = request.get_json()
    try:
        data = RefactorSnippet(**data)
        refactored_code = CodebaseRefactorer.refactor_snippet(
            data.source_code, data.language, data.rules
        )
        return jsonify({"refactored_code": refactored_code})
    except (ValueError, CodebaseRefactorerException) as e:
        return jsonify({"error": str(e)}), 400


def main():
    openai.api_key = os.getenv("OPENAI_API_KEY")
    if not openai.api_key:
        sys.exit(
            "Please set the OPENAI_API_KEY environment variable to your OpenAI API key."
        )
    logger.info(f"Starting server. Listening at: http://127.0.0.1:5000")
    app.run(debug=True)


if __name__ == "__main__":
    main()
