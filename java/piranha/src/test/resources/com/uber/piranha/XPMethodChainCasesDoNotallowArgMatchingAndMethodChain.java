package com.uber.piranha;

class XPMethodChainCases {

  interface SomeParam {
    @BoolParam(key = "other_flag")
    BoolParameter otherFlag();

    static SomeParam create(Parameter cp) {
      return null;
    }
  }

  public BoolParameter staleFlag() {
    return null;
  }
  // should not match instance method where nested invocation is not a member select tree.
  public void testDontMatchNonInstanceNested() {
    // Does not Match
    if (staleFlag().getValue()) {
      System.out.print("!!");
    }
  }

  public static void foobar(Parameter cp) {
    SomeParam sp = SomeParam.create(cp);
    // Matches API
    if (sp.staleFlag().getValue()) {
      System.out.println("!");
    }
    // Matches API
    if (!sp.staleFlag().getValue()) {
      System.out.println("!!!");
    }
    // Does not match API
    if (sp.otherFlag().getValue()) {
      System.out.println("!!!");
    }
    if (sp.otherFlag().getValue() && sp.staleFlag().getValue()) {
      System.out.println("!!!");
    }
    if (sp.otherFlag().getValue() || sp.staleFlag().getValue()) {
      System.out.println("!!!");
    }
    SomeParamRev spr = SomeParamRev.create(cp);
    // Does not match API- is reverse order
    if (spr.getValue().staleFlag()) {
      System.out.println("!!!!");
    }
    // Does not match API- matches partially
    if (spr.getValue() != null) {
      System.out.println("!!!!!");
    }
    SomeOtherInterface sot = SomeOtherInterface.create(cp);
    // Does not match API- matches partially
    if (sot.staleFlag() != null) {
      System.out.println("!!");
    }
    // Does not Match - static method invocation
    if (StaticMthds.staleFlag().getValue()) {
      System.out.print("!!");
    }

    System.out.println("done!");
    // Matches API
    cp.put(sp.staleFlag(), true);
    cp.put(sp.staleFlag(), false);

    // Do not match API
    cp.put(sp.otherFlag(), true);
    cp.put(sp.otherFlag(), false);
  }

  class TestMethodChainTest {

    public void testSomethingTreated() {
      System.out.println();
    }

    // Does not match annotation
    @PVal(ns = "", key = "other_flag", val = "false")
    public void testSomethingOther() {
      System.out.println();
    }
  }
}
