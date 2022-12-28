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

import com.google.common.collect.ImmutableMap;
import com.uber.piranha.XPFlagCleaner;
import java.util.Map;
import javax.annotation.Nullable;

/** A class representing a method configuration record from properties.json */
public final class PiranhaMethodRecord extends MethodRecord {

  /**
   * Holds the mapping of flagType string to API. Eg: "treated" -> API.IS_TREATED. Is initialized
   * once and then accessed without updating.
   */
  private static final ImmutableMap<String, XPFlagCleaner.API> flagTypeToAPIMap =
      initializeFlagTypeToAPIMap();

  private final XPFlagCleaner.API apiType;

  PiranhaMethodRecord(
      String methodName,
      String flagTypeString,
      @Nullable Integer argumentIdx,
      @Nullable String receiverType,
      @Nullable String returnType,
      @Nullable Boolean isStatic) {
    super(methodName, argumentIdx, receiverType, returnType, isStatic);
    this.apiType = flagTypeToAPIMap.getOrDefault(flagTypeString, XPFlagCleaner.API.UNKNOWN);
  }

  public XPFlagCleaner.API getApiType() {
    return apiType;
  }

  private static ImmutableMap<String, XPFlagCleaner.API> initializeFlagTypeToAPIMap() {
    ImmutableMap.Builder<String, XPFlagCleaner.API> builder = new ImmutableMap.Builder<>();
    builder.put("treated", XPFlagCleaner.API.IS_TREATED);
    builder.put("control", XPFlagCleaner.API.IS_CONTROL);
    builder.put("set_treated", XPFlagCleaner.API.SET_TREATED);
    builder.put("set_control", XPFlagCleaner.API.SET_CONTROL);
    builder.put("empty", XPFlagCleaner.API.DELETE_METHOD);
    builder.put("treatmentGroup", XPFlagCleaner.API.IS_TREATMENT_GROUP_CHECK);
    return builder.build();
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
  static PiranhaMethodRecord parseFromJSONPropertyEntryMap(
      Map<String, Object> methodPropertyEntry, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    String methodName = getValueStringFromMap(methodPropertyEntry, METHOD_NAME_KEY);
    String flagType = getValueStringFromMap(methodPropertyEntry, FLAG_TYPE_KEY);
    Integer argumentIndexInteger = getArgumentIndexFromMap(methodPropertyEntry, ARGUMENT_INDEX_KEY);
    if (methodName == null) {
      throw new PiranhaConfigurationException(
          "methodProperty is missing mandatory methodName field. Check:\n" + methodPropertyEntry);
    } else if (flagType == null) {
      throw new PiranhaConfigurationException(
          "methodProperty is missing mandatory flagType field. Check:\n" + methodPropertyEntry);
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

    return new PiranhaMethodRecord(
        methodName,
        flagType,
        argumentIndexInteger,
        getValueStringFromMap(methodPropertyEntry, RECEIVER_TYPE_STRING),
        getValueStringFromMap(methodPropertyEntry, RETURN_TYPE_STRING),
        getValueBooleanFromMap(methodPropertyEntry, METHOD_IS_STATIC));
  }
}
