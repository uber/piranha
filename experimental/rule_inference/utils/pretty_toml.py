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

from toml.encoder import TomlEncoder, _dump_str, unicode


def _dump_str_prefer_multiline(v):
    multilines = v.split("\n")
    if len(multilines) > 1:
        return unicode('"""\n' + v.replace('"""', '\\"""').strip() + '\n"""')
    else:
        return _dump_str(v)


class PrettyTOML(TomlEncoder):
    def __init__(self, _dict=dict, preserve=False):
        super(PrettyTOML, self).__init__(_dict=dict, preserve=preserve)
        self.dump_funcs[str] = _dump_str_prefer_multiline
