#  Copyright (c) 2023 Uber Technologies, Inc.

#  <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
#  except in compliance with the License. You may obtain a copy of the License at
#  <p>http://www.apache.org/licenses/LICENSE-2.0

#  <p>Unless required by applicable law or agreed to in writing, software distributed under the
#  License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
#  express or implied. See the License for the specific language governing permissions and
#  limitations under the License.

import argparse
import logging

import glob

from update_calendar_interval import UpdateCalendarInterval
from IDF_model_signature_change import IDFModelSignatureChange
from accessing_execution_plan import AccessingExecutionPlan
from gradient_boost_trees import GradientBoostTrees
from calculator_signature_change import CalculatorSignatureChange
from sql_new_execution import SQLNewExecutionChange
from query_test_check_answer_change import QueryTestCheckAnswerChange
from spark_config import SparkConfigChange
from java_spark_context import JavaSparkContextChange
from scala_session_builder import ScalaSessionBuilder


def _parse_args():
    parser = argparse.ArgumentParser(
        description="Updates the codebase to use a new version of `spark3`"
    )
    parser.add_argument(
        "--path_to_codebase",
        required=True,
        help="Path to the codebase directory.",
    )
    parser.add_argument(
        "--new_version",
        default="3.3",
        help="Version of `Spark` to update to.",
    )
    args = parser.parse_args()
    return args


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)


def main():
    args = _parse_args()
    if args.new_version == "3.3":
        upgrade_to_spark_3_3(args.path_to_codebase)


def upgrade_to_spark_3_3(path_to_codebase: str):
    """Wraps calls to Piranha with try/except to prevent it failing on a single file.
    We catch `BaseException`, as pyo3 `PanicException` extends it."""
    for scala_file in glob.glob(f"{path_to_codebase}/**/*.scala", recursive=True):
        try:
            update_file(scala_file)
        except BaseException as e:
            logging.error(f"Error running for file file {scala_file}: {e}")

    for java_file in glob.glob(f"{path_to_codebase}/**/*.java", recursive=True):
        try:
            update_file(java_file)
        except BaseException as e:
            logging.error(f"Error running for file file {java_file}: {e}")


def update_file(file_path: str):
    update_calendar_interval = UpdateCalendarInterval([file_path])
    _ = update_calendar_interval()

    idf_model_signature_change = IDFModelSignatureChange([file_path])
    _ = idf_model_signature_change()

    accessing_execution_plan = AccessingExecutionPlan([file_path])
    _ = accessing_execution_plan()

    gradient_boost_trees = GradientBoostTrees([file_path])
    _ = gradient_boost_trees()

    calculator_signature_change = CalculatorSignatureChange([file_path])
    _ = calculator_signature_change()

    sql_new_execution = SQLNewExecutionChange([file_path])
    _ = sql_new_execution()

    query_test_check_answer_change = QueryTestCheckAnswerChange([file_path])
    _ = query_test_check_answer_change()

    spark_config = SparkConfigChange([file_path])
    _ = spark_config()

    spark_config = SparkConfigChange([file_path], language="java")
    _ = spark_config()

    javasparkcontext = JavaSparkContextChange([file_path], language="java")
    _ = javasparkcontext()

    scalasessionbuilder = ScalaSessionBuilder([file_path], language="scala")
    _ = scalasessionbuilder()


if __name__ == "__main__":
    main()
