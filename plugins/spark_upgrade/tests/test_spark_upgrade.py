# Copyright (c) 2023 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

import logging
from pathlib import Path
from os import walk
from tempfile import TemporaryDirectory
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

FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)


def test_update_CalendarInterval():
    input_codebase = (
        "plugins/spark_upgrade/tests/resources/update_calendar_interval/input/"
    )
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/update_calendar_interval/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        update_calendar_interval = UpdateCalendarInterval([tp])
        summary = update_calendar_interval()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_update_IDFModelSignatureChange():
    input_codebase = (
        "plugins/spark_upgrade/tests/resources/idf_model_signature_change/input/"
    )
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/idf_model_signature_change/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        idf_model_signature_change = IDFModelSignatureChange([tp])
        summary = idf_model_signature_change()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_update_accessing_execution_plan():
    input_codebase = (
        "plugins/spark_upgrade/tests/resources/accessing_execution_plan/input/"
    )
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/accessing_execution_plan/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        accessing_execution_plan = AccessingExecutionPlan([tp])
        summary = accessing_execution_plan()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_update_gradient_boost_trees():
    input_codebase = "plugins/spark_upgrade/tests/resources/gradient_boost_trees/input/"
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/gradient_boost_trees/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        gradient_boost_trees = GradientBoostTrees([tp])
        summary = gradient_boost_trees()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_update_calculator_signature_change():
    input_codebase = (
        "plugins/spark_upgrade/tests/resources/calculator_signature_change/input/"
    )
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/calculator_signature_change/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        calculator_signature_change = CalculatorSignatureChange([tp])
        summary = calculator_signature_change()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_sql_new_execution():
    input_codebase = "plugins/spark_upgrade/tests/resources/sql_new_execution/input/"
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/sql_new_execution/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        sql_new_execution = SQLNewExecutionChange([tp])
        summary = sql_new_execution()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_query_test_check_answer_change():
    input_codebase = (
        "plugins/spark_upgrade/tests/resources/query_test_check_answer_change/input/"
    )
    expected_codebase = (
        "plugins/spark_upgrade/tests/resources/query_test_check_answer_change/expected/"
    )
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        query_test_check_answer_change = QueryTestCheckAnswerChange([tp])
        summary = query_test_check_answer_change()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)


def test_spark_config_change():
    input_codebase = "plugins/spark_upgrade/tests/resources/spark_conf/input/"
    expected_codebase = "plugins/spark_upgrade/tests/resources/spark_conf/expected/"
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        spark_config_change = SparkConfigChange([tp])
        summary = spark_config_change()
        assert summary is not None
        spark_config_change = SparkConfigChange([tp], "java")
        summary = spark_config_change()
        assert summary is not None

        javasparkcontext = JavaSparkContextChange([tp], language="java")
        summary = javasparkcontext()
        assert summary is not None
        javasparkcontext = JavaSparkContextChange([tp])
        summary = javasparkcontext()
        assert summary is not None

        scalasessionbuilder = ScalaSessionBuilder([tp], language="scala")
        summary = scalasessionbuilder()
        assert summary is not None
        scalasessionbuilder = ScalaSessionBuilder([tp], language="java")
        summary = scalasessionbuilder()
        assert summary is not None

        assert is_as_expected_files(expected_codebase, tp)


def remove_whitespace(input_str):
    """Removes all the whitespace from the string.
    Args:
        input_str (str): input string
    Returns:
        str: transformed input strings with no whitespace
    """
    return "".join(input_str.split()).strip()


def test_integration():
    """Test that the integration of all plugins terminate correctly."""
    with TemporaryDirectory() as input_temp_dir:
        for input_dir in [
            "plugins/spark_upgrade/tests/resources/accessing_execution_plan/input/",
            "plugins/spark_upgrade/tests/resources/calculator_signature_change/input/",
            "plugins/spark_upgrade/tests/resources/gradient_boost_trees/input/",
            "plugins/spark_upgrade/tests/resources/idf_model_signature_change/input/",
            "plugins/spark_upgrade/tests/resources/spark_conf/input/",
            "plugins/spark_upgrade/tests/resources/sql_new_execution/input/",
            "plugins/spark_upgrade/tests/resources/update_calendar_interval/input/",
        ]:
            dir_name = input_dir.split("/")[-3]
            copy_dir(input_dir, input_temp_dir + "/" + dir_name + "/")

        update_calendar_interval = UpdateCalendarInterval([input_temp_dir])
        _ = update_calendar_interval()
        idf_model_signature_change = IDFModelSignatureChange([input_temp_dir])
        _ = idf_model_signature_change()
        accessing_execution_plan = AccessingExecutionPlan([input_temp_dir])
        _ = accessing_execution_plan()
        gradient_boost_trees = GradientBoostTrees([input_temp_dir])
        _ = gradient_boost_trees()
        calculator_signature_change = CalculatorSignatureChange([input_temp_dir])
        _ = calculator_signature_change()
        sql_new_execution = SQLNewExecutionChange([input_temp_dir])
        _ = sql_new_execution()
        query_test_check_answer_change = QueryTestCheckAnswerChange([input_temp_dir])
        _ = query_test_check_answer_change()
        spark_config = SparkConfigChange([input_temp_dir])
        _ = spark_config()
        spark_config = SparkConfigChange([input_temp_dir], language="java")
        _ = spark_config()
        javasparkcontext = JavaSparkContextChange([input_temp_dir], language="java")
        _ = javasparkcontext()
        scalasessionbuilder = ScalaSessionBuilder([input_temp_dir], language="scala")
        _ = scalasessionbuilder()


def copy_dir(source_dir, dest_dir):
    """Copy files in {source_dir} to {dest_dir}
    Properties to note:
    * Assumes {dest_dir} is present.
    * Overwrites the similarly named files.
    Args:
        source_dir (str):
        dir_name (str):
    """
    for root, _, files in walk(source_dir):
        src_root = Path(root)
        dest_root = Path(dest_dir, src_root.relative_to(source_dir))
        Path(dest_root).mkdir(parents=True, exist_ok=True)
        for f in files:
            src_file = Path(src_root, f)
            dest_file = Path(
                dest_root, f.replace(".testjava", ".java").replace(".testkt", ".kt")
            )
            dest_file.write_text(src_file.read_text())


def is_as_expected_files(path_to_expected, path_to_actual):
    for root, _, files in walk(path_to_actual):
        actual_root = Path(root)
        expected_root = Path(path_to_expected, actual_root.relative_to(path_to_actual))
        for file_name in files:
            actual_file = Path(actual_root, file_name)
            expected_file = Path(expected_root, file_name)
            if not expected_file.exists():
                expected_file = Path(
                    expected_root,
                    file_name.replace(".java", ".testjava")
                    .replace(".kt", ".testkt")
                    .replace(".swift", ".testswift")
                    .replace(".go", ".testgo"),
                )
            _actual_content = actual_file.read_text()
            actual_content = remove_whitespace(_actual_content).strip()
            expected_content = remove_whitespace(expected_file.read_text())
            if not actual_content and expected_file.exists():
                logging.error(f"Actual content of the file was empty !!!")
                return False
            if expected_content != actual_content:
                logging.error(f"Actual content of the file :\n{_actual_content}")
                return False
    return True
