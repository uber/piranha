/**
 * Copyright (c) 2019 Uber Technologies, Inc.
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

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

class XPFlagCleanerNegativeCases {

  enum TestExperimentName {
    RANDOM_FLAG
  }

  @Retention(RetentionPolicy.RUNTIME)
  @Target({ElementType.METHOD})
  @interface ToggleTesting {
    TestExperimentName[] treated();
  }

  private XPTest experimentation;

  private boolean tBool = false;

  private boolean tBool1 = false;
  private boolean tBool2 = false;

  public void conditional_contains_nonstale_flag() {
    if (experimentation.isFlagTreated(TestExperimentName.RANDOM_FLAG)) {
      System.out.println("Hello World");
    }
  }

  public void conditional_with_else_contains_nonstale_flag() {
    if (experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG)) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void complex_conditional_contains_nonstale_flag() {
    if (tBool1 || (tBool2 && experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG))) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void assignments_containing_nonstale_flag() {
    tBool = experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG);

    tBool = experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG) && tBool;

    tBool = experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG) || tBool;

    tBool = experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG) && (tBool1 || tBool2);
  }

  public boolean return_contains_nonstale_flag() {
    return experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG);
  }

  public void condexp_contains_nonstale_flag() {
    tBool = experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG) ? true : false;
  }

  public void misc_xp_apis_containing_nonstale_flag() {
    if (experimentation.isToggleEnabled(TestExperimentName.RANDOM_FLAG) && (tBool1 || tBool2)) {}

    experimentation.putToggleEnabled(TestExperimentName.RANDOM_FLAG);

    experimentation.putToggleDisabled(TestExperimentName.RANDOM_FLAG);

    if (experimentation.isToggledDisabled(TestExperimentName.RANDOM_FLAG) && (tBool1 || tBool2)) {}
  }

  @ToggleTesting(treated = TestExperimentName.RANDOM_FLAG)
  public void annotation_test() {}

  class XPTest {
    public boolean isToggleEnabled(TestExperimentName x) {
      return true;
    }

    public boolean putToggleEnabled(TestExperimentName x) {
      return true;
    }

    public boolean isToggledDisabled(TestExperimentName x) {
      return true;
    }

    public boolean isFlagTreated(TestExperimentName x) {
      return true;
    }

    public boolean putToggleDisabled(TestExperimentName x) {
      return true;
    }
  }
}
