/**
 * Copyright (c) 2019 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */

const path = require("path");

module.exports = {
  parseProperties: function (properties_json) {
    let properties;
    let properties_abs_path = path.resolve(properties_json);

    try {
      properties = require(properties_abs_path);
    } catch (err) {
      if (err instanceof SyntaxError) {
        throw new Error(`${properties_json} does not follow JSON syntax`);
      } else {
        throw new Error(`File ${properties_json} not found`);
      }
    }

    if (!("methodProperties" in properties)) {
      throw new Error(
        `The methodProperties property is missing in ${properties_json}`
      );
    } else {
      var missingMethod = properties.methodProperties.find(
        (prop) => !("methodName", "flagType", "argumentIndex" in prop)
      );
      missingMethod = JSON.stringify(missingMethod);

      if (missingMethod != null) {
        throw new Error(
          `${missingMethod} in ${properties_json} doesn't have all required keys`
        );
      }
    }

    return properties;
  },
};
