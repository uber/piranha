from collections import Counter
from os.path import join, dirname, getmtime
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
from logging import info

find_Replace_dir = join(dirname(__file__), "find_replace_custom_cleanup")


def java_demo():
    """
    Replace new `new ArrayList<>()` with `Collections.emptyList()`
    Also add the import for Collections.
    The purpose of having `AnotherClass.java` is to demonstrate that the import statement is only added
    as a consequence of the seed expression update.
    """
    info("Running the Find/Replace Custom Cleanup demo for Java")

    directory_path = join(find_Replace_dir, "java")
    file_path = join(find_Replace_dir, "java", "SomeClass.java")
    configuration_path = join(find_Replace_dir, "java/configurations")

    old_mtime = getmtime(file_path)

    args = PiranhaArguments(
        "java",
        substitutions={
            "input_type_name": "ArrayList",
        },
        path_to_configurations=configuration_path,
        paths_to_codebase=[directory_path],
    )
    _ = execute_piranha(args)

    new_mtime = getmtime(file_path)

    assert old_mtime < new_mtime


# TODO: Migrate this usage to `execute_piranha` after the deprecated `run_piranha_cli` is removed
def python_demo():
    """
    Deletes the string literal `@str_literal` (from `substitutions` in `piranha_arguments.toml`) if it appears as a list element.
    The string literal can be a single quotes or double quotes python string.
    Also deletes an assignment to an empty list if @str_literal is the only element of the list.
    The purpose of having `empty_list` is to demonstrate that the empty list assignment is only triggered as a consequence of the seed rule.
    Finally, replaces the string literal `@str_to_replace` with `@str_replacement` (from `substitutions` in `piranha_arguments.toml`) if it appears as a list element.
    """
    print("Running the Find/Replace Custom Cleanup demo for Python")

    directory_path = join(find_Replace_dir, "python")
    file_path = join(find_Replace_dir, "python", "only_lists.py")
    configuration_path = join(find_Replace_dir, "python/configurations")

    old_mtime = getmtime(file_path)

    args = PiranhaArguments(
        "py",
        substitutions={
            "str_literal": "dependency2",
            "str_to_replace": "dependency1",
            "str_replacement": "dependency1_1"
        },
        path_to_configurations=configuration_path,
        paths_to_codebase=[directory_path],
    )
    _ = execute_piranha(args)

    new_mtime = getmtime(file_path)

    assert old_mtime < new_mtime


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

java_demo()
python_demo()
info("Completed running the Find/Replace Custom Cleanup demos")
