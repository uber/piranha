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

FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)


def test_update_CalendarInterval():
    input_codebase = "plugins/spark_upgrade/tests/resources/input/update_calendar_interval/"
    expected_codebase = "plugins/spark_upgrade/tests/resources/expected/update_calendar_interval/"
    logging.info("Here")
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
        
        update_calendar_interval = UpdateCalendarInterval([tp])
        summary = update_calendar_interval()
        assert summary is not None
        assert is_as_expected_files(expected_codebase, tp)

        
def test_update_IDFModelSignatureChange():
    input_codebase = "plugins/spark_upgrade/tests/resources/input/idf_model_signature_change/"
    expected_codebase = "plugins/spark_upgrade/tests/resources/expected/idf_model_signature_change/"
    logging.info("Here")
    with TemporaryDirectory() as temp_dir:
        tp = temp_dir
        copy_dir(input_codebase, tp)
                
        idf_model_signature_change = IDFModelSignatureChange([tp])
        summary = idf_model_signature_change()
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
