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
    Rule,
)


class IDFModelSignatureChange(ExecutePiranha):
    def __init__(self, paths_to_codebase: List[str]):
        super().__init__(
            paths_to_codebase=paths_to_codebase,
            substitutions={"michelangelo_idf_model": "MichelangeloIDFModel"},
            language="scala",
        )

    def step_name(self) -> str:
        return "IDF Model Signature Change"

    def get_rules(self) -> List[Rule]:
        transform_IDFModel_args = Rule(
            name="transform_IDFModel_args",
            query="""cs feature.IDFModel(
                MichelangeloIDFModel.getVectorFromProto(:[protoTransformer].getIDF.getIdfModel)
            )""",
            replace_node="*",
            replace="""new feature.IDFModel(
                MichelangeloIDFModel.getVectorFromProto(@protoTransformer.getIDF.getIdfModel),
                new Array[Long](@protoTransformer.getTopicSimilarity.getIdfModel.getIdfModel.getSize),
                0L
            )""",
            holes = {"michelangelo_idf_model"}
        )
        return [transform_IDFModel_args]

    def summaries_to_custom_dict(self, _) -> Dict[str, Any]:
        return {}
