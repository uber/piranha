/**
 *    Copyright (c) 2019 Uber Technologies, Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */
package com.uber.piranha;

import com.google.common.collect.ImmutableList;
// BUG: Diagnostic contains: Cleans stale plugins

class PluginCleanerPositiveCases {

  enum UniPluginDeclaration {
    // BUG: Diagnostic contains: Cleans stale plugins
    StalePluginVar//[PIRANHA_DELETE_FILE_SEQ] Delete this class.
  }

  enum TestPluginDeclarations {
     // BUG: Diagnostic contains: Cleans stale plugins
     NonStalePluginVar
  }

  enum AnotherTestPluginDeclarations {
     NonStalePluginVar,
     // BUG: Diagnostic contains: Cleans stale plugins
     NonStalePluginVar2
  }

   enum YetAnotherTestPluginDeclarations {
     NonStalePluginVar,
     // BUG: Diagnostic contains: Cleans stale plugins
  }

  // BUG: Diagnostic contains: Cleans stale plugins
  static class StalePluginFactory {
    StalePluginFactory(Object o) {}

    public Object createNewPlugin() {
      // BUG: Diagnostic contains: Cleans stale plugins
      return new Integer(1);//[PIRANHA_STALE_PLUGIN_HELPER_CLASS]=Integer
    }

    public interface ParentComponent {}
  }//[PIRANHA_DELETE_FILE_SEQ] Delete this class.

  static class PluginFactory1 {

    public interface ParentComponent {}
  }

  static class PluginFactory2 {

    public interface ParentComponent {}
  }

  public void foo() {
    ImmutableList<Object> iList1 = ImmutableList.<Object>builder()
        .add("a", "b", "c")
        .build();

    ImmutableList<Object> iList2 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("b", "c")
        .build();

    ImmutableList<Object> iList3 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("a", "c")
        .build();

    ImmutableList<Object> iList4 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("a", "b")
        .build();

    ImmutableList<Object> iList5 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add(
            "a",
            "b"
        )
        .build();

    ImmutableList<Object> iList6 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .build();

    ImmutableList<Object> iList7 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("a")
        .build();

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList8 = ImmutableList.<Object>of(
        "b"
    );

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList9 = ImmutableList.<Object>of();

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList10 = ImmutableList.<Object>of(
        "b"
    );
  }

  public interface ParentComponent1 extends PluginFactory1.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent2
      extends PluginFactory1.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent3
      extends PluginFactory1.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent4
      extends PluginFactory1.ParentComponent,
      PluginFactory2.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent5 {
     void foo();
  }

  // BUG: Diagnostic contains: Cleans stale plugins
  class ParentComponent6 {
    ParentComponent6() { super(); }
    void foo() {}
  }

}
