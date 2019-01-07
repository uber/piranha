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
    System.out.println("Hello World");
  }

  public void conditional_with_else_contains_stale_flag() {
    System.out.println("Hello World");
  }

  public void complex_conditional_contains_stale_flag() {
    System.out.println("Hello World");
  }

  public void other_api_stale_flag() {
    System.out.println("Hello World");
  }

  public void assignments_containing_stale_flag() {
    tBool = true;

    tBool = true;

    tBool = true;

    tBool = true;

    tBool = true;
  }

  public boolean return_contains_stale_flag() {
    return true;
  }

  public void condexp_contains_stale_flag() {
    tBool =  true;
  }

  public void misc_xp_apis_containing_stale_flag() {  }

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
