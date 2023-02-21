from os.path import join, dirname, getmtime, exists
from polyglot_piranha import execute_piranha, PiranhaArguments
import logging
from logging import info

feature_flag_dir = join(dirname(__file__), "feature_flag_cleanup")


def run_java_ff_demo():
    info("Running the stale feature flag cleanup demo for Java")

    directory_path = join(feature_flag_dir, "java")
    modified_file_path = join(directory_path, "SampleClass.java")
    deleted_join_path = join(directory_path, "TestEnum.java")
    configuration_path = join(directory_path, "configurations")

    old_mtime = getmtime(modified_file_path)

    args = PiranhaArguments(
        "java",
        {
            "stale_flag_name": "SAMPLE_STALE_FLAG",
            "treated": "true",
            "treated_complement": "false",
        },
        path_to_codebase=directory_path,
        path_to_configurations=configuration_path,
    )
    output_summary_java = execute_piranha(args)

    assert len(output_summary_java) == 2

    for summary in output_summary_java:
        assert len(summary.rewrites) > 0

    new_mtime = getmtime(modified_file_path)

    assert old_mtime < new_mtime
    assert not exists(deleted_join_path)


def run_kt_ff_demo():
    info("Running the stale feature flag cleanup demo for Kotlin")

    directory_path = join(feature_flag_dir, "kt")
    modified_file_path = join(directory_path, "SampleClass.kt")
    deleted_join_path = join(directory_path, "TestEnum.kt")
    configuration_path = join(directory_path, "configurations")

    old_mtime = getmtime(modified_file_path)

    args = PiranhaArguments(
        "kt",
        {
            "stale_flag_name": "SAMPLE_STALE_FLAG",
            "treated": "true",
            "treated_complement": "false",
        },
        path_to_codebase=directory_path,
        path_to_configurations=configuration_path,
    )

    output_summary_kt = execute_piranha(args)

    assert len(output_summary_kt) == 2

    for summary in output_summary_kt:
        assert len(summary.rewrites) > 0

    new_mtime = getmtime(modified_file_path)

    assert old_mtime < new_mtime
    assert not exists(deleted_join_path)


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

run_java_ff_demo()
run_kt_ff_demo()
print("Completed running the stale feature flag cleanup demos")
