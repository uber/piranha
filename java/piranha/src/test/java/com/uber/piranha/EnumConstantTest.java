/**
 * Copyright (c) 2021 Uber Technologies, Inc.
 *
 * <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 * <p>http://www.apache.org/licenses/LICENSE-2.0
 *
 * <p>Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.uber.piranha;

import static com.uber.piranha.PiranhaTestingHelpers.addExperimentFlagEnumsWithConstructor;

import com.google.errorprone.BugCheckerRefactoringTestHelper;
import com.google.errorprone.ErrorProneFlags;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

/**
 * This test suite tests Piranha logic relating to enum constants with custom constructors with
 * arguments that match on a flag value
 *
 * <p>See: <a href="https://github.com/uber/piranha/issues/141">Issue #141</a>
 */
@RunWith(JUnit4.class)
public class EnumConstantTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  /**
   * Test to check that Java enum constants are removed when they have a constructor with an
   * argument that matches the flag value, for an untreated flag
   *
   * <p>Also tests flags that use unqualified enum constant usages, and enum method calls
   */
  @Test
  public void testRemoveEnumFieldValueMatchUntreated() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, true); // Adds STALE_FLAG, etc enums

    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class PositiveCaseEnumTest {",
            "  private XPTest experimentation;",
            "  public void printFlags() {",
            "    if (experimentation.isToggleEnabled(STALE_FLAG)) {",
            "      System.out.println(\"Stale Flag\");",
            "    }",
            "    if (experimentation.isToggleDisabled(STALE_FLAG.getKey())) {",
            "      System.out.println(\"Stale Flag getKey\");",
            "    }",
            "    if (experimentation.isToggleEnabled(OTHER_FLAG)) {",
            "      System.out.println(\"Other Flag\");",
            "    }",
            "    if (experimentation.isToggleDisabled(OTHER_FLAG.getKey())) {",
            "      System.out.println(\"Other Flag getKey\");",
            "    }",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "class PositiveCaseEnumTest {",
            "  private XPTest experimentation;",
            "  public void printFlags() {",
            "    System.out.println(\"Stale Flag getKey\");",
            "    if (experimentation.isToggleEnabled(OTHER_FLAG)) {",
            "      System.out.println(\"Other Flag\");",
            "    }",
            "    if (experimentation.isToggleDisabled(OTHER_FLAG.getKey())) {",
            "      System.out.println(\"Other Flag getKey\");",
            "    }",
            "  }",
            "}")
        .doTest();
  }

  /**
   * Test to check that Java enum constants are removed when they have a constructor with an
   * argument that matches the flag value, for a treated flag
   *
   * <p>Also tests flags that use qualified enum constant usages, and enum method calls
   */
  @Test
  public void testRemoveEnumFieldValueMatchTreated() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, true); // Adds STALE_FLAG, etc enums

    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class PositiveCaseEnumTest {",
            "  private XPTest experimentation;",
            "  public void printFlags() {",
            "    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {",
            "      System.out.println(\"Stale Flag\");",
            "    }",
            "    if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG.getKey())) {",
            "      System.out.println(\"Stale Flag getKey\");",
            "    }",
            "    if (experimentation.isToggleEnabled(TestExperimentName.OTHER_FLAG)) {",
            "      System.out.println(\"Other Flag\");",
            "    }",
            "    if (experimentation.isToggleEnabled(TestExperimentName.OTHER_FLAG.getKey())) {",
            "      System.out.println(\"Other Flag getKey\");",
            "    }",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "class PositiveCaseEnumTest {",
            "  private XPTest experimentation;",
            "  public void printFlags() {",
            "    System.out.println(\"Stale Flag\");",
            "    System.out.println(\"Stale Flag getKey\");",
            "    if (experimentation.isToggleEnabled(TestExperimentName.OTHER_FLAG)) {",
            "      System.out.println(\"Other Flag\");",
            "    }",
            "    if (experimentation.isToggleEnabled(TestExperimentName.OTHER_FLAG.getKey())) {",
            "      System.out.println(\"Other Flag getKey\");",
            "    }",
            "  }",
            "}")
        .doTest();
  }

  /**
   * Test to check that Java enum constants are removed when they have a constructor with an
   * argument that matches the flag value, for an untreated flag, in test code
   *
   * <p>Also tests flags that use unqualified enum constant usages, and enum method calls
   */
  @Test
  public void testRemoveEnumFieldValueMatchTest() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, true); // Adds STALE_FLAG, etc enums

    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     System.err.println(\"To be removed\");",
            "  }",
            "  @Test",
            "  public void test_StaleFlag_treated_methodCall() {",
            "     experimentation.putToggleEnabled(STALE_FLAG.getKey());",
            "     System.err.println(\"To be removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated_methodCall() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG.getKey());",
            "     System.err.println(\"Not removed\");",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated_methodCall() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG.getKey());",
            "     System.err.println(\"Not removed\");",
            "  }",
            "}")
        .doTest();
  }
}
