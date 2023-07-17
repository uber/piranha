# Copyright (c) 2023 Uber Technologies, Inc.
#
# <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
# except in compliance with the License. You may obtain a copy of the License at
# <p>http://www.apache.org/licenses/LICENSE-2.0
#
# <p>Unless required by applicable law or agreed to in writing, software distributed under the
# License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
# express or implied. See the License for the specific language governing permissions and
# limitations under the License.

import attr
import toml

from piranha_playground.rule_inference.utils.pretty_toml import PrettyTOML
from piranha_playground.rule_inference.utils.rule_utils import RawRuleGraph

LANGUAGES = ["kt", "java", "go", "swift"]


@attr.s
class InferData:
    """
    Data class representing the information needed for the infer_piranha event.
    """

    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(LANGUAGES))


@attr.s
class ImproveData:
    """
    Data class representing the information needed for the improve_piranha event.
    """

    source_code = attr.ib(validator=attr.validators.instance_of(str))
    target_code = attr.ib(validator=attr.validators.instance_of(str))
    language = attr.ib(validator=attr.validators.in_(LANGUAGES))
    requirements = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)
    option = attr.ib(validator=attr.validators.in_(["user", "general"]))

    def __attrs_post_init__(self):
        self.rules = toml.dumps(toml.loads(self.rules), encoder=PrettyTOML())


@attr.s
class RefactorData:
    """
    Data class representing the information needed for the refactor_codebase event.
    """

    language = attr.ib(validator=attr.validators.in_(LANGUAGES))
    folder_path = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)


@attr.s
class RefactorSnippet:
    """
    Data class representing the information needed for the refactor_codebase event.
    """

    language = attr.ib(validator=attr.validators.in_(LANGUAGES))
    source_code = attr.ib(validator=attr.validators.instance_of(str))
    rules = attr.ib(validator=RawRuleGraph.validate)
