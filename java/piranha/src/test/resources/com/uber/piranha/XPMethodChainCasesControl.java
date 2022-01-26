package com.uber.piranha;

class XPMethodChainCases {

  interface SomeParam {
    @BoolParam(key = "other_flag")
    BoolParameter otherFlag();

    static SomeParam create(Parameter cp) {
      return null;
    }
  }

  public static void foobar(Parameter cp) {
    SomeParam sp = SomeParam.create(cp);

    System.out.println("!!!");

    SomeParamRev spr = SomeParamRev.create(cp);

    if (spr.getValue().staleFlag()) {
      System.out.println("!!!!");
    }

    if (spr.getValue() != null) {
      System.out.println("!!!!!");
    }
    SomeOtherInterface sot = SomeOtherInterface.create(cp);

    if (sot.staleFlag() != null) {
      System.out.println("!!");
    }
    System.out.println("done!");

    cp.put("", "other_flag", true);
    cp.put("", "other_flag", false);
  }

  class TestMethodChainTest {

    public void testSomethingControl() {
      System.out.println();
    }

    @PVal(ns = "", key = "other_flag", val = "false")
    public void testSomethingOther() {
      System.out.println();
    }
  }

  interface SomeOtherInterface {
    SomeParam staleFlag();

    static SomeOtherInterface create(Parameter cp) {
      return null;
    }
  }

  interface OverlappingNameInterface {
    public boolean staleFlag();

    static OverlappingNameInterface create(Parameter cp) {
      return null;
    }
  }

  interface SomeParamRev {
    OverlappingNameInterface getValue();

    static SomeParamRev create(Parameter cp) {
      return null;
    }
  }
}
