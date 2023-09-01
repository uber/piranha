# Copyright (c) 2023 Uber Technologies, Inc.

# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0

# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

from typing import Any, List, Dict
from execute_piranha import ExecutePiranha

from polyglot_piranha import (
    Filter,
    OutgoingEdges,
    Rule,
)


class UpdateCalendarInterval(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={"calendarInterval": "CalendarInterval"},
            language="scala",
        )

    def step_name(self) -> str:
        return "Update CalendarInterval.:[x]"

    def get_rules(self) -> List[Rule]:
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
        return [update_CalendarInterval, add_import_DateTimeConstants]

    def get_edges(self) -> List[OutgoingEdges]:
        return [
            OutgoingEdges(
                "update_CalendarInterval",
                to=["add_import_DateTimeConstants"],
                scope="File",
            )
        ]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
