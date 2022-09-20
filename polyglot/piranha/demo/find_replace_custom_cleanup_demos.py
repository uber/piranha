from collections import Counter
from os.path import join, dirname
from polyglot_piranha import run_piranha_cli
import logging 
from logging import info 

find_Replace_dir = join(dirname(__file__), 'find_replace_custom_cleanup')

def java_demo():
    """
    Replace new `new ArrayList<>()` with `Collections.emptyList()`
    Also add the import for Collections.
    The purpose of having `AnotherClass.java` is to demonstrate that the import statement is only added 
    as a consequence of the seed expression update.
    """    
    info("Running the Find/Replace Custom Cleanup demo for Java")
    _ = run_piranha_cli(join(find_Replace_dir, "java"), join(find_Replace_dir, "java/configurations"), True)


FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)

java_demo()
info("Completed running the Find/Replace Custom Cleanup demos")
