from collections import Counter
from os.path import join, dirname
from polyglot_piranha import run_piranha_cli
import logging
from logging import info

match_only_dir = join(dirname(__file__), 'match_only')

def demo():
    info("Running the Match-only demo for Java")
    output_summary_java = run_piranha_cli(join(match_only_dir, "java"), join(match_only_dir, "java/configurations"), True)

    rule_match_counter = Counter([m[0] for m in output_summary_java[0].matches])

    assert rule_match_counter['find_fooBar_anywhere'] == 2


    assert rule_match_counter['find_barFoo_in_static_method'] == 1

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

demo()
info("Completed running the Match-only demo")
