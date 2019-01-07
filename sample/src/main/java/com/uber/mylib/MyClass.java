package com.uber.mylib;

/** A sample class. */
public class MyClass {

  enum TestExperimentName {
    SAMPLE_STALE_FLAG
  }

  private XPTest expt;

  public void foo() {
    if(expt.flagEnabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
        System.out.println("Hello World");
    }
  }

  public void bar() {
    if(expt.flagDisabled(TestExperimentName.SAMPLE_STALE_FLAG)) {
        System.out.println("Hi World");
    }
  }

  static class XPTest {
    public boolean flagEnabled(TestExperimentName x) { return true; }
    public boolean enableFlag(TestExperimentName x) { return true; }
    public boolean disableFlag(TestExperimentName x) { return true; }
    public boolean flagDisabled(TestExperimentName x) { return true; }
  }


}
