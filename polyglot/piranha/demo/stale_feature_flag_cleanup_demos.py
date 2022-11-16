from os.path import join, dirname, getmtime
from polyglot_piranha import run_piranha_cli
import logging 
from logging import info 

feature_flag_dir = join(dirname(__file__), 'feature_flag_cleanup')

def run_java_ff_demo():
    info("Running the stale feature flag cleanup demo for Java")

    directory_path = join(feature_flag_dir, "java")
    file_paths = [
        join(directory_path, "SampleClass.java"),
        join(directory_path, "TestEnum.java"),
    ]
    configuration_path = join(directory_path, "configurations")

    old_mtimes = list(getmtime(file_path) for file_path in file_paths)

    output_summary_java = run_piranha_cli(directory_path, configuration_path, False)

    assert len(output_summary_java) == 2

    for summary in output_summary_java:
        assert len(summary.rewrites) > 0

    new_mtimes = list(getmtime(file_path) for file_path in file_paths)

    for i in range(len(old_mtimes)):
        assert old_mtimes[i] < new_mtimes[i]

def run_kt_ff_demo():
    info("Running the stale feature flag cleanup demo for Kotlin")

    directory_path = join(feature_flag_dir, "kt")
    file_paths = [
        join(directory_path, "SampleClass.kt"),
        join(directory_path, "TestEnum.kt"),
    ]
    configuration_path = join(directory_path, "configurations")

    old_mtimes = list(getmtime(file_path) for file_path in file_paths)

    output_summary_kt = run_piranha_cli(directory_path, configuration_path, False)
    
    assert len(output_summary_kt) == 2

    for summary in output_summary_kt:
        assert len(summary.rewrites) > 0

    new_mtimes = list(getmtime(file_path) for file_path in file_paths)

    for i in range(len(old_mtimes)):
        assert old_mtimes[i] < new_mtimes[i]

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

run_java_ff_demo()
run_kt_ff_demo()
print("Completed running the stale feature flag cleanup demos")
