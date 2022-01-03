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

import java.util.Map;
import javax.annotation.Nullable;

/** Utility methods used by {@link PiranhaEnumRecord} and {@link PiranhaMethodRecord} classes */
public class PiranhaRecord {
  /**
   * Utility method. Checks whether the value associated to a given map and given key is a non-empty
   * string.
   *
   * @param map - map corresponding to a method property
   * @param key - key to check the corresponding value
   * @return String if value is a non-empty string, null otherwise
   */
  @Nullable
  static String getValueStringFromMap(Map<String, Object> map, String key) {
    Object value = map.get(key);
    if (value instanceof String) {
      String valueStr = String.valueOf(value);
      if (!valueStr.isEmpty()) return valueStr;
    }
    return null;
  }

  /**
   * Utility method. Checks whether the argumentIndex key of a method property map is a non-negative
   * integer.
   *
   * @param map - map corresponding to a method property
   * @return argumentIndex if argument index is a non-negative integer, null otherwise
   */
  @Nullable
  static Integer getArgumentIndexFromMap(Map<String, Object> map, String argumentIndexKey) {
    Object value = map.get(argumentIndexKey);
    if (value instanceof Long) {
      int argumentIndex = ((Long) value).intValue();
      if (argumentIndex >= 0) {
        return argumentIndex;
      }
    }
    return null;
  }

  /**
   * Utility method. Checks whether the value associated to a given map and given key is a non-empty
   * string.
   *
   * @param map - map corresponding to a method property
   * @param key - key to check the corresponding value
   * @return the corresponding boolean value
   */
  @Nullable
  static Boolean getValueBooleanFromMap(Map<String, Object> map, String key) {
    Object value = map.get(key);
    if (value instanceof String && !value.equals("")) {
      return Boolean.parseBoolean(String.valueOf(value));
    }
    return null;
  }
}
