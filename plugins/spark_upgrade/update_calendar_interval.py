from typing import List

from polyglot_piranha import (
    Filter,
    OutgoingEdges,
    PiranhaArguments,
    Rule,
    RuleGraph,
    execute_piranha,
)


def update_CalendarInterval(paths_to_codebase: List[str]):
    update_CalendarInterval = Rule(
        name="update_CalendarInterval",
        query="cs CalendarInterval.:[x]",
        replace_node="*",
        replace="DateTimeConstants.@x",
        holes={"calendarInterval"},
    )
    add_import_DateTimeConstants = Rule(
        name="add_import_DateTimeConstants",
        query="(package_clause) @package_clause",
        replace_node="package_clause",
        replace="@package_clause\nimport org.apache.spark.sql.catalyst.util.DateTimeConstants",
        filters={
            Filter(
                enclosing_node="(compilation_unit) @cu",
                not_contains=[
                    "cs import org.apache.spark.sql.catalyst.util.DateTimeConstants"
                ],
            )
        },
        is_seed_rule=False,
    )

    args = PiranhaArguments(
        language="scala",
        paths_to_codebase=paths_to_codebase,
        substitutions={"calendarInterval": "CalendarInterval"},
        rule_graph=RuleGraph(
            rules=[update_CalendarInterval, add_import_DateTimeConstants],
            edges=[
                OutgoingEdges(
                    "update_CalendarInterval",
                    to=["add_import_DateTimeConstants"],
                    scope="File",
                )
            ],
        ),
        allow_dirty_ast=True,
    )

    return execute_piranha(args)
