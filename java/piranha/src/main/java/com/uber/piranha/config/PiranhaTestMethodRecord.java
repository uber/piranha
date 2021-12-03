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

import static com.uber.piranha.config.PiranhaMethodRecord.parseArgumentIndexFromJSON;
import static com.uber.piranha.config.PiranhaMethodRecord.parseMethodNameFromJSON;
import static com.uber.piranha.config.PiranhaRecord.getValueStringFromMap;

import java.util.Map;
import java.util.Optional;

/** A class representing a test method configuration record from properties.json */
public final class PiranhaTestMethodRecord implements PiranhaMethodRecord {
  // Used for generating exception messages
  private static final String PROPERTY_NAME = "testMethodProperty";

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

    String methodName = parseMethodNameFromJSON(PROPERTY_NAME, methodPropertyEntry);
    Integer argumentIndexInteger =
        parseArgumentIndexFromJSON(PROPERTY_NAME, methodPropertyEntry, isArgumentIndexOptional);
    String receiverType = getValueStringFromMap(methodPropertyEntry, RECEIVER_TYPE_STRING);
    String returnType = getValueStringFromMap(methodPropertyEntry, RETURN_TYPE_STRING);

    return new PiranhaTestMethodRecord(
        methodName,
        Optional.ofNullable(argumentIndexInteger),
        Optional.ofNullable(receiverType),
        Optional.ofNullable(returnType));
  }
}
