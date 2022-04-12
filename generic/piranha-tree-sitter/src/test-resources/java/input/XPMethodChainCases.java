package com.uber.piranha;

class XPMethodChainCases {

  // should not match instance method where nested invocation is not a member select tree.
  public void testDontMatchNonInstanceNested() {
    // Does not Match
    if (isStaleFeature().getCachedValue()) {
      System.out.print("!!");
    }
  }

  public static void foobar(Parameter cp) {
    SomeParam sp = SomeParam.create(cp);
    // Matches API
    if (sp.isStaleFeature().getCachedValue()) {
      System.out.println("!");
    }
    // Matches API
    if (!sp.isStaleFeature().getCachedValue()) {
      System.out.println("!!!");
    }
    // Does not match API
    if (sp.otherFlag().getCachedValue()) {
      System.out.println("!!!");
    }
    if (sp.otherFlag().getCachedValue() && sp.isStaleFeature().getCachedValue()) {
      System.out.println("!!!");
    }
    if (sp.otherFlag().getCachedValue() || sp.isStaleFeature().getCachedValue()) {
      System.out.println("!!!");
    }
    SomeParamRev spr = SomeParamRev.create(cp);
    // Does not match API- is reverse order
    if (spr.getCachedValue().isStaleFeature()) {
      System.out.println("!!!!");
    }
    // Does not match API- matches partially
    if (spr.getCachedValue() != null) {
      System.out.println("!!!!!");
    }
    SomeOtherInterface sot = SomeOtherInterface.create(cp);
    // Does not match API- matches partially
    if (sot.isStaleFeature() != null) {
      System.out.println("!!");
    }
    // Does not Match - static method invocation
    if (StaticMthds.isStaleFeature().getCachedValue()) {
      System.out.print("!!");
    }

    System.out.println("done!");
    // Matches API
    cp.put(sp.isStaleFeature(), true);
    cp.put(sp.isStaleFeature(), false);

    // Do not match API
    cp.put(sp.otherFlag(), true);
    cp.put(sp.otherFlag(), false);
  }
}