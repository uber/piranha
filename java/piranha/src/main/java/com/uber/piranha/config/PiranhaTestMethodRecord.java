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

/** A class representing a test method configuration record from properties.json */
public final class PiranhaTestMethodRecord implements PiranhaMethodRecord {

  // Allowed fields for a test method property in the config file.
  // Entered under the top-level "testMethodProperties" in properties.json.
  // By default, methodName and argumentIndex fields are mandatory.
  // The returnType and receiverType fields are optional.
  private static final String METHOD_NAME_KEY = "methodName";
  private static final String ARGUMENT_INDEX_KEY = "argumentIndex";
  private static final String RETURN_TYPE_STRING = "returnType";
  private static final String RECEIVER_TYPE_STRING = "receiverType";

  private final String methodName;
  private final Optional<Integer> argumentIdx;
  private final Optional<String> receiverType;
  private final Optional<String> returnType;

  PiranhaTestMethodRecord(
      String methodName,
      Optional<Integer> argumentIdx,
      Optional<String> receiverType,
      Optional<String> returnType) {
    this.methodName = methodName;
    this.argumentIdx = argumentIdx;
    this.receiverType = receiverType;
    this.returnType = returnType;
  }

  @Override
  public String getMethodName() {
    return methodName;
  }

  @Override
  public Optional<Integer> getArgumentIdx() {
    return argumentIdx;
  }

  @Override
  public Optional<String> getReceiverType() {
    return receiverType;
  }

  @Override
  public Optional<String> getReturnType() {
    return returnType;
  }

  /**
   * Parse the entry for a single method from piranha.json that has been previously decoded into a
   * map
   *
   * @param methodPropertyEntry The decoded json entry (as a Map of property names to values)
   * @param isArgumentIndexOptional Whether argumentIdx should be treated as optional
   * @return A PiranhaTestMethodRecord corresponding to the given map/json record.
   * @throws PiranhaConfigurationException if there was any issue reading or parsing the
   *     configuration file.
   */
  static PiranhaTestMethodRecord parseFromJSONPropertyEntryMap(
      Map<String, Object> methodPropertyEntry, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    String methodName = getValueStringFromMap(methodPropertyEntry, METHOD_NAME_KEY);
    Integer argumentIndexInteger = getArgumentIndexFromMap(methodPropertyEntry, ARGUMENT_INDEX_KEY);
    if (methodName == null) {
      throw new PiranhaConfigurationException(
          "testMethodProperty is missing mandatory methodName field. Check:\n"
              + methodPropertyEntry);
    } else if (!isArgumentIndexOptional && argumentIndexInteger == null) {
      throw new PiranhaConfigurationException(
          "testMethodProperty did not have argumentIndex. By default, Piranha requires an argument index for flag "
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

    return new PiranhaTestMethodRecord(
        methodName,
        Optional.ofNullable(argumentIndexInteger),
        Optional.ofNullable(getValueStringFromMap(methodPropertyEntry, RECEIVER_TYPE_STRING)),
        Optional.ofNullable(getValueStringFromMap(methodPropertyEntry, RETURN_TYPE_STRING)));
  }
}
