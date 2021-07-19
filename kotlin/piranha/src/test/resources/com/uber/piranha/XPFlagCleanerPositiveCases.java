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

import dagger.Provides;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import javax.inject.Inject;

class XPFlagCleanerPositiveCases {

  enum TestExperimentName {
    // BUG: Diagnostic contains: Cleans stale XP flags
    STALE_FLAG
  }

  enum AnotherTestExperimentName {
    // BUG: Diagnostic contains: Cleans stale XP flags
    @Autorollout
    STALE_FLAG
  }

  @Retention(RetentionPolicy.RUNTIME)
  public @interface Autorollout {
    boolean staged() default false;
  }

  @Retention(RetentionPolicy.RUNTIME)
  @Target({ElementType.METHOD})
  @interface ToggleTesting {
    TestExperimentName[] treated();
  }

  private XPTest experimentation;

  private boolean tBool = false;

  public void conditional_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
      System.out.println("Hello World");
    }
  }

  public void conditional_with_else_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void complex_conditional_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (true || (tBool && experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG))) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void other_api_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isFlagTreated(TestExperimentName.STALE_FLAG)) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void assignments_containing_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && true;

    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || true;

    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || tBool;

    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true);
  }

  public boolean return_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    return experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG);
  }

  public void condexp_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) ? true : false;
  }

  public void misc_xp_apis_containing_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {}

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.putToggleEnabled(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.includeEvent(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.putToggleDisabled(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {}
  }

  public int return_within_if_basic() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
      return 20;
    }
    return 30;
  }

  public int return_within_if_additional(int x) {
    if (x == 0) {
      // BUG: Diagnostic contains: Cleans stale XP flags
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        return 0;
      }
      return 75;
    }

    if (x == 1)
      // BUG: Diagnostic contains: Cleans stale XP flags
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        return 1;
      } else {
        return 76;
      }

    if (x == 2) {
      int y = 3;
      // BUG: Diagnostic contains: Cleans stale XP flags
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        y++;
        return y;
      }
      return y + 10;
    }

    if (x == 3) {
      int z = 4;
      // BUG: Diagnostic contains: Cleans stale XP flags
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        z++;
      } else {
        z = z * 5;
        return z + 10;
      }
      return z;
    }

    return 100;
  }

  @ToggleTesting(treated = TestExperimentName.STALE_FLAG)
  // BUG: Diagnostic contains: Cleans stale XP flags
  public void annotation_test() {}

  @Inject XPTest injectedExperimentsShouldBeDeleted;

  private int testRemovingInjectField() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (injectedExperimentsShouldBeDeleted.isToggleEnabled(TestExperimentName.STALE_FLAG)) return 1;
    else return 2;
  }

  @Inject XPTest injectedExperimentsMultipleUses;

  private void randomSet(XPTest x) {
    injectedExperimentsMultipleUses = x;
  }

  private int testNotRemovingInjectField() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (injectedExperimentsMultipleUses.isToggleEnabled(TestExperimentName.STALE_FLAG)) return 1;
    else return 2;
  }

  @Provides
  public int unusedParamTestWithDeletion(XPTest x) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (x.isToggleEnabled(TestExperimentName.STALE_FLAG)) return 1;
    else return 2;
  }

  @Provides
  public int unusedParamTestWithoutDeletion(XPTest x) {

    if (x != null) {
      // just another use to prevent deletion of this parameter.
    }

    // BUG: Diagnostic contains: Cleans stale XP flags
    if (x.isToggleEnabled(TestExperimentName.STALE_FLAG)) return 1;
    else return 2;
  }

  private void testMultipleCalls(int x) {
    if (x > 0) {
      // BUG: Diagnostic contains: Cleans stale XP flags
      experimentation.includeEvent(TestExperimentName.STALE_FLAG);
      // BUG: Diagnostic contains: Cleans stale XP flags
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        // comment0
        return;
      } else {
        // comment1
        return;
      }
    }

    // do something here
    return;
  }

  public int or_compounded_with_not(int x, boolean extra_toggle) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (extra_toggle || !experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
      return 0;
    } else {
      return 1;
    }
  }

  public int remove_else_if(boolean extra_toggle) {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if (extra_toggle) {
      return 0;
    } else if (experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
      return 1;
    } else {
      return 2;
    }
  }

  class XPTest {
    public boolean isToggleEnabled(TestExperimentName x) {
      return true;
    }

    public boolean putToggleEnabled(TestExperimentName x) {
      return true;
    }

    public boolean includeEvent(TestExperimentName x) {
      return true;
    }

    public boolean isToggleDisabled(TestExperimentName x) {
      return true;
    }

    public boolean putToggleDisabled(TestExperimentName x) {
      return true;
    }

    public boolean isFlagTreated(TestExperimentName x) {
      return true;
    }

    public boolean isToggleInGroup(TestExperimentName x) {
      return true;
    }
  }
}
