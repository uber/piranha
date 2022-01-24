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
import static com.uber.piranha.config.PiranhaRecord.getValueBooleanFromMap;
import static com.uber.piranha.config.PiranhaRecord.getValueStringFromMap;

import java.util.Map;
import java.util.Optional;
import javax.annotation.Nullable;

/** A class representing a method configuration record from properties.json */
public class MethodRecord {

  // Allowed fields for a method property in the config file.
  // Entered under the top-level "methodProperties" in properties.json.
  // By default, the flagType, methodName and argumentIndex fields are mandatory.
  // The returnType and receiverType fields are optional.
  protected static final String FLAG_TYPE_KEY = "flagType";
  protected static final String METHOD_NAME_KEY = "methodName";
  protected static final String ARGUMENT_INDEX_KEY = "argumentIndex";
  protected static final String RETURN_TYPE_STRING = "returnType";
  protected static final String RECEIVER_TYPE_STRING = "receiverType";
  protected static final String METHOD_IS_STATIC = "isStatic";

  /**
   * Holds the mapping of flagType string to API. Eg: "treated" -> API.IS_TREATED. Is initialized
   * once and then accessed without updating.
   */
  private final String methodName;

  @Nullable private final Integer argumentIdx;
  @Nullable private final String receiverType;
  @Nullable private final String returnType;
  private final Boolean isStatic;

  MethodRecord(
      String methodName,
      @Nullable Integer argumentIdx,
      @Nullable String receiverType,
      @Nullable String returnType,
      @Nullable Boolean isStatic) {
    this.methodName = methodName;
    this.argumentIdx = argumentIdx;
    this.receiverType = receiverType;
    this.returnType = returnType;
    this.isStatic = isStatic != null && isStatic;
  }

  public String getMethodName() {
    return methodName;
  }

  public Optional<Integer> getArgumentIdx() {
    return Optional.ofNullable(argumentIdx);
  }

  public Optional<String> getReceiverType() {
    return Optional.ofNullable(receiverType);
  }

  public Optional<String> getReturnType() {
    return Optional.ofNullable(returnType);
  }

  /**
   * Parse the entry for a single method from piranha.json that has been previously decoded into a
   * map
   *
   * @param methodPropertyEntry The decoded json entry (as a Map of property names to values)
   * @param isArgumentIndexOptional Whether argumentIdx should be treated as optional
   * @return A PiranhaMethodRecord corresponding to the given map/json record.
   * @throws PiranhaConfigurationException if there was any issue reading or parsing the
   *     configuration file.
   */
  static MethodRecord parseFromJSONPropertyEntryMap(
      Map<String, Object> methodPropertyEntry, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    String methodName = getValueStringFromMap(methodPropertyEntry, METHOD_NAME_KEY);
    Integer argumentIndexInteger = getArgumentIndexFromMap(methodPropertyEntry, ARGUMENT_INDEX_KEY);
    if (methodName == null) {
      throw new PiranhaConfigurationException(
          "methodProperty is missing mandatory methodName field. Check:\n" + methodPropertyEntry);
    } else if (!isArgumentIndexOptional && argumentIndexInteger == null) {
      throw new PiranhaConfigurationException(
          "methodProperty did not have argumentIndex. By default, Piranha requires an argument index for flag "
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

    return new MethodRecord(
        methodName,
        argumentIndexInteger,
        getValueStringFromMap(methodPropertyEntry, RECEIVER_TYPE_STRING),
        getValueStringFromMap(methodPropertyEntry, RETURN_TYPE_STRING),
        getValueBooleanFromMap(methodPropertyEntry, METHOD_IS_STATIC));
  }

  public boolean isStatic() {
    return isStatic;
  }
}
