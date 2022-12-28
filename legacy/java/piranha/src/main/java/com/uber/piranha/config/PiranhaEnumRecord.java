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

/** A class representing an enum configuration record from properties.json */
public final class PiranhaEnumRecord {

  // Allowed fields for a enum property in the config file.
  // Entered under the top-level "enumProperties" in properties.json.
  // By default, enumName and argumentIndex fields are mandatory.
  private static final String ENUM_NAME_KEY = "enumName";
  private static final String ARGUMENT_INDEX_KEY = "argumentIndex";

  private final String enumName;
  private final Optional<Integer> argumentIdx;

  PiranhaEnumRecord(String enumName, Optional<Integer> argumentIdx) {
    this.enumName = enumName;
    this.argumentIdx = argumentIdx;
  }

  public String getEnumName() {
    return enumName;
  }

  public Optional<Integer> getArgumentIdx() {
    return argumentIdx;
  }

  /**
   * Parse the entry for a single enum from piranha.json that has been previously decoded into a map
   *
   * @param enumPropertyEntry The decoded json entry (as a Map of property names to values)
   * @param isArgumentIndexOptional Whether argumentIdx should be treated as optional
   * @return A PiranhaEnumPropertyRecord corresponding to the given map/json record.
   * @throws PiranhaConfigurationException if there was any issue reading or parsing the
   *     configuration file.
   */
  static PiranhaEnumRecord parseFromJSONPropertyEntryMap(
      Map<String, Object> enumPropertyEntry, boolean isArgumentIndexOptional)
      throws PiranhaConfigurationException {
    String enumName = getValueStringFromMap(enumPropertyEntry, ENUM_NAME_KEY);
    Integer argumentIndexInteger = getArgumentIndexFromMap(enumPropertyEntry, ARGUMENT_INDEX_KEY);
    if (enumName == null) {
      throw new PiranhaConfigurationException(
          "enumProperty is missing mandatory enumName field. Check:\n" + enumPropertyEntry);
    } else if (!isArgumentIndexOptional && argumentIndexInteger == null) {
      throw new PiranhaConfigurationException(
          "enumProperty did not have argumentIndex. By default, Piranha requires an argument index for flag "
              + "APIs, to which the flag name/symbol will be passed. This is to avoid over-deletion of all "
              + "occurrences of a flag API method. If you are sure you want to delete all instances of the "
              + "method below, consider using Piranha:ArgumentIndexOptional=true to override this behavior. "
              + "Check:\n"
              + enumPropertyEntry);
    } else if (argumentIndexInteger != null && argumentIndexInteger < 0) {
      throw new PiranhaConfigurationException(
          "Invalid argumentIndex field. Arguments are zero indexed. Check:\n" + enumPropertyEntry);
    }

    return new PiranhaEnumRecord(enumName, Optional.ofNullable(argumentIndexInteger));
  }
}
