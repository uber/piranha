from os.path import join, dirname, getmtime, exists
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
from logging import info

feature_flag_dir = dirname(__file__)


def clean_flag(flag_name: str, directory: str):
    info("Running the stale feature flag cleanup demo for Java")

    directory_path = join(feature_flag_dir, directory)
    configuration_path = join(feature_flag_dir, "piranha_config")
    print(configuration_path)

    args = PiranhaArguments(
        "java",
        substitutions={
            "arg": flag_name,
            "val": "true"
        },
        path_to_codebase=directory_path,
        path_to_configurations=configuration_path,

    )
    output_summary_java = execute_piranha(args)
    print(output_summary_java)


def log():
    FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
    logging.basicConfig(format=FORMAT)
    logging.getLogger().setLevel(logging.DEBUG)



clean_flag('"*.disable_villagers"', "repos")

