/**
 * Copyright (c) 2021 Uber Technologies, Inc.
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
package com.uber.piranha.config;

import static com.uber.piranha.config.PiranhaRecord.getArgumentIndexFromMap;
import static com.uber.piranha.config.PiranhaRecord.getValueStringFromMap;

import java.util.Map;
import java.util.Optional;

/** An interface representing a method configuration record from properties.json */
public interface PiranhaMethodRecord {
  // Allowed fields for a method property in the config file.
  // Entered under the top-level "methodProperties" or "testMethodProperties" in properties.json.
  // By default, the flagType, methodName and argumentIndex fields are mandatory.
  // The returnType and receiverType fields are optional.
  String FLAG_TYPE_KEY = "flagType";
  String METHOD_NAME_KEY = "methodName";
  String ARGUMENT_INDEX_KEY = "argumentIndex";
  String RETURN_TYPE_STRING = "returnType";
  String RECEIVER_TYPE_STRING = "receiverType";

  String getMethodName();

  Optional<Integer> getArgumentIdx();

  Optional<String> getReceiverType();

  Optional<String> getReturnType();

  static String parseMethodNameFromJSON(
      String propertyName, Map<String, Object> methodPropertyEntry)
      throws PiranhaConfigurationException {
    String methodName = getValueStringFromMap(methodPropertyEntry, METHOD_NAME_KEY);
    if (methodName == null) {
      throw new PiranhaConfigurationException(
          propertyName + " is missing mandatory methodName field. Check:\n" + methodPropertyEntry);
    }
    return methodName;
  }

  static String parseFlagTypeFromJSON(String propertyName, Map<String, Object> methodPropertyEntry)
      throws PiranhaConfigurationException {
    String flagType = getValueStringFromMap(methodPropertyEntry, FLAG_TYPE_KEY);
    if (flagType == null) {
      throw new PiranhaConfigurationException(
          propertyName + " is missing mandatory flagType field. Check:\n" + methodPropertyEntry);
    }
    return flagType;
  }

  static Integer parseArgumentIndexFromJSON(
      String propertyName, Map<String, Object> methodPropertyEntry, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    Integer argumentIndexInteger = getArgumentIndexFromMap(methodPropertyEntry, ARGUMENT_INDEX_KEY);
    if (!isArgumentIndexOptional && argumentIndexInteger == null) {
      throw new PiranhaConfigurationException(
          propertyName
              + " did not have argumentIndex. By default, Piranha requires an argument index for flag "
              + "APIs, to which the flag name/symbol will be passed. This is to avoid over-deletion of all "
              + "occurrences of a flag API method. If you are sure you want to delete all instances of the "
              + "method below, consider using Piranha:ArgumentIndexOptional=true to override this behavior. "
              + "Check:\n"
              + methodPropertyEntry);
    } else if (argumentIndexInteger != null && argumentIndexInteger < 0) {
      throw new PiranhaConfigurationException(
          "Invalid argumentIndex field. Arguments are zero indexed. Check:\n"
              + methodPropertyEntry);
    }
    return argumentIndexInteger;
  }
}
