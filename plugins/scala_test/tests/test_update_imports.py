from logging import error
from pathlib import Path
from os.path import join, basename
from os import listdir
from update_imports import update_imports

def test_update_imports():
    summary = update_imports("plugins/scala_test/tests/resources/input/", "3.2.2", dry_run=True)
    assert is_as_expected("plugins/scala_test/tests/resources/", summary)

def is_as_expected(path_to_scenario, output_summary):
    expected_output = join(path_to_scenario, "expected")
    print("Summary", output_summary)
    input_dir = join(path_to_scenario, "input")
    for file_name in listdir(expected_output):
        with open(join(expected_output, file_name), "r") as f:
            file_content = f.read()
            expected_content = "".join(file_content.split())

            # Search for the file in the output summary
            updated_content = [
                "".join(o.content.split())
                for o in output_summary
                if basename(o.path) == file_name
            ]
            print(file_name)
            # Check if the file was rewritten
            if updated_content:
                if expected_content != updated_content[0]:
                    error("----update" + updated_content[0] )
                    return False
            else:
                # The scenario where the file is not expected to be rewritten
                original_content= Path(join(input_dir, file_name)).read_text()
                if expected_content != "".join(original_content.split()):
                    return False
    return True
