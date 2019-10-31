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
import com.uber.piranha.PluginCleanerPositiveCases.StalePluginFactory;

class PluginCleanerPositiveCases {

  enum UniPluginDeclaration {
    // BUG: Diagnostic contains: Cleans stale plugins
    StalePluginVar
  }

  enum TestPluginDeclarations {
     // BUG: Diagnostic contains: Cleans stale plugins
     StalePluginVar,
     NonStalePluginVar
  }

  enum AnotherTestPluginDeclarations {
     NonStalePluginVar,
     // BUG: Diagnostic contains: Cleans stale plugins
     StalePluginVar,
     NonStalePluginVar2
  }

  enum YetAnotherTestPluginDeclarations {
     NonStalePluginVar,
     // BUG: Diagnostic contains: Cleans stale plugins
     StalePluginVar
  }

  // BUG: Diagnostic contains: Cleans stale plugins
  static class StalePluginFactory {
    StalePluginFactory(Object o) {}

    public Object createNewPlugin() {
      // BUG: Diagnostic contains: Cleans stale plugins
      return new Integer(1);
    }

    public interface ParentComponent {}
  }

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
        .add(new StalePluginFactory("a"), "b", "c")
        .build();

    ImmutableList<Object> iList3 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("a", new StalePluginFactory("a"), "c")
        .build();

    ImmutableList<Object> iList4 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add("a", "b", new StalePluginFactory("a"))
        .build();

    ImmutableList<Object> iList5 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add(
            "a",
            "b",
            new StalePluginFactory("a")
        )
        .build();

    ImmutableList<Object> iList6 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add(new StalePluginFactory("a"))
        .build();

    ImmutableList<Object> iList7 = ImmutableList.<Object>builder()
        // BUG: Diagnostic contains: Cleans stale plugins
        .add(new StalePluginFactory("a"))
        .add("a")
        .build();

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList8 = ImmutableList.<Object>of(
        new StalePluginFactory("a"),
        "b"
    );

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList9 = ImmutableList.<Object>of(
        new StalePluginFactory("a")
    );

    // BUG: Diagnostic contains: Cleans stale plugins
    ImmutableList<Object> iList10 = ImmutableList.<Object>of(
        new StalePluginFactory("a"),
        "b"
    );
  }

  public interface ParentComponent1 extends PluginFactory1.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent2
      extends PluginFactory1.ParentComponent,
      StalePluginFactory.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent3
      extends StalePluginFactory.ParentComponent,
      PluginFactory1.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent4
      extends PluginFactory1.ParentComponent,
      StalePluginFactory.ParentComponent,
      PluginFactory2.ParentComponent {}

  // BUG: Diagnostic contains: Cleans stale plugins
  public interface ParentComponent5
      extends StalePluginFactory.ParentComponent {
     void foo();
  }

  // BUG: Diagnostic contains: Cleans stale plugins
  class ParentComponent6
      implements StalePluginFactory.ParentComponent {
    ParentComponent6() { super(); }
    void foo() {}
  }
}
