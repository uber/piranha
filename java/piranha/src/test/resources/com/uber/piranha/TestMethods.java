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

class MockObject {
  public boolean thenReturn(boolean bool) {
    return bool;
  }
}

class TestMethods {
  public static boolean mockable() {
    return false;
  }

  public static MockObject mock(Object thingToMock) {
    return new MockObject();
  }

  public static MockObject keepMe(Object thingToMock) {
    return new MockObject();
  }

  public static MockObject expect(Object thingToMock) {
    return new MockObject();
  }

  public static MockObject expect(boolean value, Object thingToMock) {
    return new MockObject();
  }
}
