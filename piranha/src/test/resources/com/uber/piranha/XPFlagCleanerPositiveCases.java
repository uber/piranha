package com.uber.piranha;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;


class XPFlagCleanerPositiveCases {

  enum TestExperimentName {
    STALE_FLAG
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
    if(experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        System.out.println("Hello World");
    }
  }

  public void conditional_with_else_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if(experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
      System.out.println("Hello World");
    } else {
      System.out.println("Hi world");
    }
  }

  public void complex_conditional_contains_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if(true || (tBool && experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG))) {
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
    tBool =  experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) ? true : false;
  }

  public void misc_xp_apis_containing_stale_flag() {
    // BUG: Diagnostic contains: Cleans stale XP flags
    if(experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool ||
        true)) {}

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.putToggleEnabled(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.includeEvent(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    experimentation.putToggleDisabled(TestExperimentName.STALE_FLAG);

    // BUG: Diagnostic contains: Cleans stale XP flags
    if(experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG) && (tBool ||
        true)) {}


  }

  @ToggleTesting(treated = TestExperimentName.STALE_FLAG)
  // BUG: Diagnostic contains: Cleans stale XP flags
  public void annotation_test() {}

  class XPTest {
    public boolean isToggleEnabled(TestExperimentName x) { return true; }
    public boolean putToggleEnabled(TestExperimentName x) { return true; }
    public boolean includeEvent(TestExperimentName x) { return true; }
    public boolean isToggleDisabled(TestExperimentName x) { return true; }
    public boolean putToggleDisabled(TestExperimentName x) { return true; }
    public boolean isFlagTreated(TestExperimentName x) { return true; }
  }


}
