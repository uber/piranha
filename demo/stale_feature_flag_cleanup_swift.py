from os.path import join, dirname, getmtime, exists
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
from logging import info

feature_flag_dir = join(dirname(__file__), "feature_flag_cleanup")


def run_swift_ff_demo():
    info("Running the stale feature flag cleanup demo for swift")

    directory_path = join(feature_flag_dir, "swift")
    modified_file_path = join(directory_path, "SampleClass.swift")
    configuration_path = join(directory_path, "configurations")

    old_mtime = getmtime(modified_file_path)

    args = PiranhaArguments(
        directory_path,
        configuration_path,
        "swift",
        {
            "stale_flag": "stale_flag"
        },
    )

    output_summary_swift = execute_piranha(args)


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

run_swift_ff_demo()
print("Completed running the stale feature flag cleanup demos")
