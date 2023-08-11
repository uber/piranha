import logging

from update_imports import update_imports

FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)



update_imports("/Users/ketkara/repositories/open-source/piranha/tests/scala_test/input/sample.scala")


