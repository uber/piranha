from os.path import join, dirname
from polyglot_piranha import run_piranha_cli

dirname = dirname(__file__)

output_summary_java = run_piranha_cli(join(dirname, 'java/ff'), join(dirname, 'java/ff/configurations'), True)


output_summary_kt = run_piranha_cli(join(dirname, 'kt/ff'), join(dirname, 'kt/ff/configurations'), True)


print()