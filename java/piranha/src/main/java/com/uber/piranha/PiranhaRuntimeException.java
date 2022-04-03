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

/**
 * An exception thrown during the execution of the Piranha bug checker. This is a runtime exception
 * and will crash piranha, as the tool has detected that it will clean up code in an invalid state.
 */
public final class PiranhaRuntimeException extends RuntimeException {

  public PiranhaRuntimeException(String message) {
    super(message);
  }
}
