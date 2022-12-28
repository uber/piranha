package com.uber.mylib;

// DO *NOT* MODIFY THIS FILE INSIDE main/java/src/...

// This file is copied automatically from main/resources/com/uber/mylib/MyClass.bak before and after
// every build, as part of the Piranha integration tests (the idea is not to leave the "cleaned"
// file lying around after Piranha runs, as that pollutes the git staging area).
// If you need to change this file, change the copy inside main/resources/com/uber/mylib/MyClass.bak

// After clean-up, this file must match main/resources/com/uber/mylib/MyClass.expect

/** A sample class. */
public class MyClass {

  enum TestExperimentName {
    SAMPLE_STALE_FLAG
  }

  private XPTest expt;

  public void foo() {
    if (expt.flagEnabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
      System.out.println("Hello World");
    }
  }

  public void bar() {
    if (expt.flagDisabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
      System.out.println("Hi World");
    }
  }

  static class XPTest {
    public boolean flagEnabled(TestExperimentName x) {
      return true;
    }

    public boolean enableFlag(TestExperimentName x) {
      return true;
    }

    public boolean disableFlag(TestExperimentName x) {
      return true;
    }

    public boolean flagDisabled(TestExperimentName x) {
      return true;
    }
  }
}
