from collections import Counter
from os.path import join, dirname
from polyglot_piranha import run_piranha_cli

find_Replace_dir = join(dirname(__file__), 'find_replace_custom_cleanup')

def java_demo():
    """
    Replace new `new ArrayList<>()` with `Collections.emptyList()`
    Also add the import for Collections.
    The purpose of having `AnotherClass.java` is to demonstrate that the import statement is only added 
    as a consequence of the seed expression update.
    """    
    print("Running the Find/Replace Custom Cleanup demo for Java")
    _ = run_piranha_cli(join(find_Replace_dir, "java"), join(find_Replace_dir, "java/configurations"), True)

def go_demo():
    """
    Replace `fmt.Print("String\n")` with `fmt.Println("String")`.
    We define a rule to replace `fmt.Print` with `fmt.Println` which then triggers the cleanup of the trailing `\n` on a matching `call_expression`.
    """
    print("Running the Find/Replace Custom Cleanup demo for Go")
    _ = run_piranha_cli(join(find_Replace_dir, "go"), join(find_Replace_dir, "go/configurations"), True)

java_demo()
go_demo()
print("Completed running the Find/Replace Custom Cleanup demos")
