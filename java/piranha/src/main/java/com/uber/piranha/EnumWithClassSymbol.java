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
package com.uber.piranha;

import com.sun.tools.javac.code.Symbol;
import java.util.Objects;

class EnumWithClassSymbol {
  private final String enumName;
  private final Symbol.ClassSymbol enumClass;

  EnumWithClassSymbol(String enumName, Symbol.ClassSymbol enumClass) {
    this.enumName = enumName;
    this.enumClass = enumClass;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (!(o instanceof EnumWithClassSymbol)) return false;
    EnumWithClassSymbol enumWithClassSymbol = (EnumWithClassSymbol) o;
    return enumName.equals(enumWithClassSymbol.enumName)
        && enumClass.equals(enumWithClassSymbol.enumClass);
  }

  @Override
  public int hashCode() {
    return Objects.hash(enumName, enumClass);
  }
}
