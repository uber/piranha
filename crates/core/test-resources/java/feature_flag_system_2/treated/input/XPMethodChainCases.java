/*
Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

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
      System.out.println("simplify and keep the statement");
    }
    // test for identifier || true
    if (a || sp.isStaleFeature().getCachedValue()){
      System.out.println("!!!");
    }
    // test for identifier && false
    if (b && !sp.isStaleFeature().getCachedValue()){
      System.out.println("!!! b");
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

  class TestMethodChainTest {
    // Matches annotation
    @ParameterValue(ns = "some_long_name", key = "STALE_FLAG", val = "true")
    public void testSomethingTreated() {
      System.out.println();
    }

    // Matches annotation
    @ParameterValue(ns = "some_long_name", key = "STALE_FLAG", val = "false")
    public void testSomethingControl() {
      System.out.println();
    }

    // Does not match annotation
    @ParameterValue(ns = "some_long_name", key = "other_flag", val = "false")
    public void testSomethingOther() {
      System.out.println();
    }
  }
}
