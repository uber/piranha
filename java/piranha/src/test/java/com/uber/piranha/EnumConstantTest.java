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
   * Test for cleaning up the first enum constant in a list of enum constants.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveFirstEnumConstant() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG,",
            " OTHER_FLAG_1,",
            " OTHER_FLAG_2;",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1,",
            " OTHER_FLAG_2;",
            "}")
        .doTest();
  }

  /**
   * Test for cleaning up an enum in the middle of the list of enum constants, matching on the enum
   * constant.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveMiddleEnumConstant() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1,",
            " STALE_FLAG,",
            " OTHER_FLAG_2;",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1,",
            " OTHER_FLAG_2;",
            "}")
        .doTest();
  }

  /**
   * Test for cleaning up an enum in the middle of the list of enum constants, matching on the
   * constructor argument value.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveMiddleEnumConstantWithNonConstantMembers() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1(\"other.1\"),",
            " STALE_FLAG(\"stale.flag\"),",
            " OTHER_FLAG_2(\"other.2\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1(\"other.1\"),",
            " OTHER_FLAG_2(\"other.2\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .doTest();
  }

  /**
   * Test for cleaning up an enum in the middle of the list of enum constants, matching on the
   * constructor argument value, however the argumentIndex is not provided
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveMiddleEnumConstantWithNonConstantMembersNoArgIndex() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_enum_no_arg.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentNameNoArg.java",
            "package com.uber.piranha;",
            "public enum TestExperimentNameNoArg {",
            " OTHER_FLAG_1(1,\"other.1\"),",
            " STALE_FLAG(1,\"stale.flag\"),",
            " OTHER_FLAG_2(1,\"other.2\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentNameNoArg(int x, final String key) {",
            "  this.key = x > 1 ? key : key+\"1\";",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentNameNoArg.java",
            "package com.uber.piranha;",
            "public enum TestExperimentNameNoArg {",
            " OTHER_FLAG_1(1,\"other.1\"),",
            " OTHER_FLAG_2(1,\"other.2\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentNameNoArg(int x, final String key) {",
            "  this.key = x > 1 ? key : key+\"1\";",
            " }",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant is removed on an enum with a custom constructor, matching
   * on the enum constant.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumConstantWithNonConstantMembersByEnumConstant() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG(\"other\"),",
            " STALE_FLAG(\"stale.flag\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG(\"other\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant is removed on an enum with a custom constructor, matching
   * on the constructor argument value.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumConstantWithNonConstantMembersByConstructorValue() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG(\"other\"),",
            " STALE_FLAG(\"stale.flag\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG(\"other\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant is removed, and there are comments between enum constants.
   *
   * <p>Piranha doesn't know if it can associate the prior comment with the stale flag, so to be
   * safe, we leave it.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumConstantWithNonConstantMembersWithComments() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " // This is another flag!",
            " OTHER_FLAG(\"other\"),",
            " // This flag is stale!",
            " STALE_FLAG(\"stale.flag\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " // This is another flag!",
            " OTHER_FLAG(\"other\");",
            // Piranha doesn't know if it can associate this comment with the stale flag, so to be
            // safe, we leave it
            " // This flag is stale!",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant is removed, and has no trailing character.
   *
   * <p>We match to the pre-existing pattern of the removed constant and the remaining constant has
   * its comma removed.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumConstantWithoutTrailingCharacter() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG,",
            " STALE_FLAG",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant with an unnecessary trailing comma is removed.
   *
   * <p>Even though the comma is not necessary, we match to the pre-existing pattern of the removed
   * enum
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumConstantWithTrailingComma() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG,",
            " STALE_FLAG,",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            // Matches the same pattern as the previous last enum
            " OTHER_FLAG,",
            "}")
        .doTest();
  }

  /**
   * Test for when an enum constant which is not last with an unnecessary trailing semicolon is
   * removed.
   *
   * <p>The semicolon on the last enum constant should be preserved.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveFirstEnumConstantWithUnneededTrailingSemicolon() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG,",
            " OTHER_FLAG;",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG;",
            "}")
        .doTest();
  }

  /**
   * Test for when the last enum constant with an unnecessary trailing semicolon is removed.
   *
   * <p>Even though the semicolon is not necessary, we match to the pre-existing pattern of the
   * removed enum
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumWithUnneededTrailingSemicolon() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG,",
            " STALE_FLAG;",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            // Matches the same pattern as the previous last enum
            " OTHER_FLAG;",
            "}")
        .doTest();
  }

  /**
   * Test to see if a trailing semicolon on an enum constant that does require it, remains
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveLastEnumWithNeededTrailingSemicolon() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG,",
            " STALE_FLAG;",
            " TestExperimentName() {",
            "  System.out.println(\"Hello World\");",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG;",
            " TestExperimentName() {",
            "  System.out.println(\"Hello World\");",
            " }",
            "}")
        .doTest();
  }

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

  /** Test to cleanup an enum with only one remaining enum constant */
  @Test
  public void testRemoveEnumWithSingleConstant() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup an enum with only one remaining enum constant, where that constant ends in a
   * comma
   */
  @Test
  public void testRemoveEnumWithSingleConstantWithComma() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG,",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup an enum with only one remaining enum constant, where that constant ends in a
   * semi-colon
   */
  @Test
  public void testRemoveEnumWithSingleConstantWithSemicolon() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG;",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup an enum with only one remaining enum constant to be cleaned up in an enum with
   * fields or methods other than enum constants. We do not remove the enum class, but we make sure
   * to leave a semi-colon before the non-enum-constant fields and methods to keep the class as
   * valid compilable Java code
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/143">Issue #143</a>
   */
  @Test
  public void testRemoveEnumWithSingleConstantWithNonEnumConstantMembers() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG;",
            " private String key;",
            " TestExperimentName() {",
            "  this.key = \"Hello World\";",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " ;",
            " private String key;",
            " TestExperimentName() {",
            "  this.key = \"Hello World\";",
            " }",
            "}")
        .doTest();
  }

  /** This is to make sure we don't confuse enum constant constructors with other constructors */
  @Test
  public void testRemoveEnumWithNonEnumConstantConstructor() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG,",
            " STALE_FLAG;",
            " private Object obj = new Object();",
            " TestExperimentName() {",
            " }",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG;",
            " private Object obj = new Object();",
            " TestExperimentName() {",
            " }",
            "}")
        .doTest();
  }

  /** Test to cleanup an enum in annotation. Should remove the entire line */
  @Test
  public void testRemoveEnumFromAnnotationShouldRemoveAnnotation() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = TestExperimentName.STALE_FLAG)",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup the first enum in annotation. Should remove the enum and the comma after it and
   * keep the annotation and the other enum
   */
  @Test
  public void testRemoveFirstEnumFromAnnotationRemovingCommaAndKeepingOtherEnum() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.STALE_FLAG, TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup the enum in the middle of the annotation. Should remove the enum and the comma
   * after it and keep the annotation and the other enums
   */
  @Test
  public void testRemoveEnumInTheMiddleFromAnnotationRemovingCommaAndKeepingOthersEnums() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG, TestExperimentName.STALE_FLAG, TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG, TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup the enum in the middle of the annotation without space between the annotation.
   * Should remove the enum and the comma after it and keep the annotation and the other enums
   */
  @Test
  public void
      testRemoveEnumInTheMiddleWithoutSpacesFromAnnotationRemovingCommaAndKeepingOthersEnums() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG,TestExperimentName.STALE_FLAG,TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG,TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup the enum in the middle of the annotation separate the enums by new line. Should
   * remove the enum and the comma after it and keep the annotation and the other enums
   */
  @Test
  public void
      testRemoveEnumInTheMiddleSeparateByNewLineFromAnnotationRemovingCommaAndKeepingOthersEnums() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG, TestExperimentName.STALE_FLAG,\r\nTestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG,\r\nTestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }

  /**
   * Test to cleanup the enum at the end of the annotation. Should remove the enum and the comma
   * before it and keep the annotation and the other enums
   */
  @Test
  public void testRemoveEnumAtTheEndFromAnnotationRemovingCommaAndKeepingOthersEnums() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG,",
            "  STALE_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG, TestExperimentName.OTHER_FLAG, TestExperimentName.STALE_FLAG})",
            " public void annotation_test() {}",
            "}")
        .addOutputLines(
            "TestEnumInAnnotation.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "class TestEnumInAnnotation {",
            " enum TestExperimentName {",
            "  OTHER_FLAG,",
            "  ANOTHER_FLAG;",
            " }",
            " @Retention(RetentionPolicy.RUNTIME)",
            " @Target({ElementType.METHOD})",
            " @interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            " }",
            " @ToggleTesting(treated = {TestExperimentName.ANOTHER_FLAG, TestExperimentName.OTHER_FLAG})",
            " public void annotation_test() {}",
            "}")
        .doTest();
  }
}
