from os.path import join, dirname
from polyglot_piranha import run_piranha_cli
import logging 
from logging import info 

feature_flag_dir = join(dirname(__file__), 'feature_flag_cleanup')

def run_java_ff_demo():
    info("Running the stale feature flag cleanup demo for Java")
    output_summary_java = run_piranha_cli(join(feature_flag_dir, "java"), join(feature_flag_dir, "java/configurations"), True)

    assert len(output_summary_java) == 2

    for summary in output_summary_java:
        assert len(summary.rewrites) > 0

def run_kt_ff_demo():
    info("Running the stale feature flag cleanup demo for Kotlin")
    output_summary_kt = run_piranha_cli(join(feature_flag_dir, "kt"), join(feature_flag_dir, 'kt/configurations'), True)
    assert len(output_summary_kt) == 2

    for summary in output_summary_kt:
        assert len(summary.rewrites) > 0

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

run_java_ff_demo()
run_kt_ff_demo()
print("Completed running the stale feature flag cleanup demos")
