from collections import Counter
from os.path import join, dirname, getmtime
from polyglot_piranha import run_piranha_cli, execute_piranha, PiranhaArguments
import logging
from logging import info

find_Replace_dir = join(dirname(__file__), "find_replace")


def swift_demo():
    """
    This shows how we can use Piranha to execute structural find/replace that hook on the the
    pre-built rules.
    """
    info("Running the Find/Replace demo for Swift")

    file_path = join(find_Replace_dir, "swift", "Sample.swift")
    configuration_path = join(find_Replace_dir, "swift/configurations")

    old_mtime = getmtime(file_path)

    args = PiranhaArguments(
        file_path,
        configuration_path,
        "swift",
        {
            "stale_flag_name": "test_second_experiment",
        },
        cleanup_comments=True,
    )

    _ = execute_piranha(args)

    new_mtime = getmtime(file_path)

    assert old_mtime < new_mtime


def java_demo():
    """
    This shows how we can use Piranha to execute structural find/replace that hook on the the
    pre-built rules.
    Note how it deletes the enum declaration and consequently the file.
    Deletion of the file can be disabled by setting the `delete_file_if_empty` flag to False.
    """
    info("Running the Find/Replace demo for Java")

    file_path = join(find_Replace_dir, "java", "TestEnum.java")
    configuration_path = join(find_Replace_dir, "java/configurations")

    old_mtime = getmtime(file_path)

    args = PiranhaArguments(
        file_path,
        configuration_path,
        "java",
        {
            "stale_flag_name": "STALE_FLAG",
        },
        cleanup_comments=True,
    )

    _ = run_piranha_cli(file_path, configuration_path, False)

    new_mtime = getmtime(file_path)

    assert old_mtime < new_mtime


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)
swift_demo()
strings_demo()
java_demo()
print("Completed running the Find/Replace demos")
