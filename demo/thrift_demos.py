from collections import Counter
from os.path import join, dirname, getmtime
from polyglot_piranha import Rule, RuleGraph, execute_piranha, PiranhaArguments, Filter
import logging
from logging import info

find_Replace_dir = join(dirname(__file__), "find_replace")


def thrift_demo():

    file_path = join(find_Replace_dir, "thrift", "sample.thrift")
    r1 = Rule(
        name="Match Exception Definition",
        query="""(
        (exception_definition (identifier) @exception_name) @exception_definition
        (#match? @exception_name "@input_exception_name")
        )""",
        replace_node="exception_definition",
        replace="""@exception_definition(
    rpc.code = "@rpc_code"
)""",
        filters={
            Filter(matcher="(exception_definition) @c_e",
                   not_contains=["(annotation_definition) @ad"])
        },
        holes={
            "input_exception_name", "rpc_code"
        }
    )
    error_code_mapping = {"Invalid": "INVALID_ARGUMENT",
                          "BadRequest": "INVALID_ARGUMENT",
                          "UserNotAllowed": "INVALID_ARGUMENT",
                          "Precondition": "FAILED_PRECONDITION",
                          "Range": "OUT_OF_RANGE",
                          "Unauthenticated": "UNAUTHENTICATED",
                          "Permission": "PERMISSION_DENIED",
                          "NotFound": "NOT_FOUND",
                          "DoesNotExist": "NOT_FOUND",
                          "Aborted": "ABORTED",
                          "Exists": "ALREADY_EXISTS",
                          "Exhausted": "RESOURCE_EXHAUSTED",
                          "Cancelled": "CANCELLED",
                          "DataLoss": "DATA_LOSS",
                          "UnknownError": "UNKNOWN",
                          "UnknownException": "UNKNOWN",
                          "InternalError": "INTERNAL",
                          "InternalException": "INTERNAL",
                          "ServiceError": "INTERNAL",
                          "ServiceException": "INTERNAL",
                          "ServerError": "INTERNAL",
                          "ServerException": "INTERNAL",
                          "Unimplemented": "UNIMPLEMENTED",
                          "Unavailable": "UNAVAILABLE",
                          "DeadlineExceeded": "DEADLINE_EXCEEDED",
                          }
    for k, v in error_code_mapping.items():
        args = PiranhaArguments(
            language="thrift",
            paths_to_codebase=[file_path],
            rule_graph=RuleGraph(rules=[r1], edges=[]),
            substitutions={"input_exception_name": k, "rpc_code": v}
        )
        output = execute_piranha(args)
        print(output)


FORMAT = "%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s"
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)
thrift_demo()
print("Completed running Thrift refactoring")
