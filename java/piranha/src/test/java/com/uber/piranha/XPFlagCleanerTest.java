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

import static com.google.common.truth.Truth.assert_;

import com.google.errorprone.BugCheckerRefactoringTestHelper;
import com.google.errorprone.CompilationTestHelper;
import com.google.errorprone.ErrorProneFlags;
import java.io.IOException;
import java.text.ParseException;
import java.util.Arrays;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class XPFlagCleanerTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private CompilationTestHelper compilationHelper;

  @Before
  public void setup() {
    compilationHelper = CompilationTestHelper.newInstance(XPFlagCleaner.class, getClass());
    compilationHelper.setArgs(Arrays.asList("-d", temporaryFolder.getRoot().getAbsolutePath()));
  }

  @Test
  public void test_xpflagsPositiveCases() {
    compilationHelper.setArgs(
        Arrays.asList(
            "-d",
            temporaryFolder.getRoot().getAbsolutePath(),
            "-XepOpt:Piranha:FlagName=STALE_FLAG",
            "-XepOpt:Piranha:IsTreated=true",
            "-XepOpt:Piranha:Config=config/piranha.properties"));
    compilationHelper.addSourceFile("XPFlagCleanerPositiveCases.java").doTest();
  }

  @Test
  public void test_xpflagsNegativeCases() {
    compilationHelper.setArgs(
        Arrays.asList(
            "-d",
            temporaryFolder.getRoot().getAbsolutePath(),
            "-XepOpt:Piranha:FlagName=STALE_FLAG",
            "-XepOpt:Piranha:IsTreated=true",
            "-XepOpt:Piranha:Config=config/piranha.properties"));
    compilationHelper.addSourceFile("XPFlagCleanerNegativeCases.java").doTest();
  }

  @Test
  public void positiveTreatment() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
          BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      BugCheckerRefactoringTestHelper.ExpectOutput eo =
          bcr.addInput("XPFlagCleanerPositiveCases.java");
      eo.addOutput("XPFlagCleanerPositiveCasesTreatment.java");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void positiveSpecificTreatmentGroup() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:TreatmentGroup", "GROUP_A");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
          BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);

      bcr.addInputLines(
              "TestExperimentName.java",
              "package com.uber.piranha;",
              "public enum TestExperimentName {",
              " STALE_FLAG",
              "}")
          .addOutputLines(
              "TestExperimentName.java",
              "package com.uber.piranha;",
              "public enum TestExperimentName {", // Ideally we would remove this too, fix later
              "}")
          .addInputLines(
              "TestExperimentGroups.java",
              "package com.uber.piranha;",
              "public enum TestExperimentGroups {",
              " GROUP_A,",
              " GROUP_B,",
              "}")
          .addOutputLines(
              "TestExperimentGroups.java",
              "package com.uber.piranha;",
              "//[PIRANHA_DELETE_FILE_SEQ] Delete this class if not automatically removed.",
              "enum TestExperimentGroups { }")
          .addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
              "import static com.uber.piranha.TestExperimentGroups.GROUP_A;",
              "import static com.uber.piranha.TestExperimentGroups.GROUP_B;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public String groupToString() {",
              "  // BUG: Diagnostic contains: Cleans stale XP flags",
              "  if (experimentation.isToggleDisabled(STALE_FLAG)) { return \"\"; }",
              "  else if (experimentation.isToggleInGroup(",
              "            STALE_FLAG,GROUP_A)) { ",
              "    return \"A\";",
              "  } else if (experimentation.isToggleInGroup(",
              "            STALE_FLAG,GROUP_B)) { ",
              "    return \"B\";",
              "  } else { return \"C\"; }",
              " }",
              "}")
          .addOutputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public String groupToString() {",
              "  return \"A\";",
              " }",
              "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void dontRemoveGenericallyNamedTreatmentGroups() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:TreatmentGroup", "TREATED");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
              BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);

      bcr.addInputLines(
              "TestExperimentName.java",
              "package com.uber.piranha;",
              "public enum TestExperimentName {",
              " STALE_FLAG",
              "}")
              .addOutputLines(
                      "TestExperimentName.java",
                      "package com.uber.piranha;",
                      "public enum TestExperimentName {", // Ideally we would remove this too, fix later
                      "}")
              .addInputLines(
                      "TestExperimentGroups.java",
                      "package com.uber.piranha;",
                      "public enum TestExperimentGroups {",
                      " TREATED,",
                      " CONTROL,",
                      "}")
              .addOutputLines(
                      "TestExperimentGroups.java",
                      "package com.uber.piranha;",
                      "public enum TestExperimentGroups {",
                      " TREATED,",
                      " CONTROL,",
                      "}")
              .addInputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
                      "import static com.uber.piranha.TestExperimentGroups.TREATED;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      " private XPTest experimentation;",
                      " public String groupToString() {",
                      "  // BUG: Diagnostic contains: Cleans stale XP flags",
                      "  if (experimentation.isToggleDisabled(STALE_FLAG)) { return \"\"; }",
                      "  else if (experimentation.isToggleInGroup(",
                      "            STALE_FLAG,TREATED)) { ",
                      "    return \"Treated\";",
                      "  } else { return \"Controll\"; }",
                      " }",
                      "}")
              .addOutputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "import static com.uber.piranha.TestExperimentGroups.TREATED;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      " private XPTest experimentation;",
                      " public String groupToString() {",
                      "  return \"Treated\";",
                      " }",
                      "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void positiveControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
          BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      BugCheckerRefactoringTestHelper.ExpectOutput eo =
          bcr.addInput("XPFlagCleanerPositiveCases.java");
      eo.addOutput("XPFlagCleanerPositiveCasesControl.java");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  private BugCheckerRefactoringTestHelper addHelperClasses(BugCheckerRefactoringTestHelper bcr)
      throws IOException {
    return bcr.addInput("XPTest.java").expectUnchanged();
  }

  @Test
  public void positiveRemoveImport() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
          BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);

      bcr.addInputLines(
              "TestExperimentName.java",
              "package com.uber.piranha;",
              "public enum TestExperimentName {",
              " STALE_FLAG",
              "}")
          .addOutputLines(
              "TestExperimentName.java",
              "package com.uber.piranha;",
              "public enum TestExperimentName {", // Ideally we would remove this too, fix later
              "}")
          .addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "import static com.uber.piranha.TestExperimentName"
                  + ".STALE_FLAG;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public boolean return_contains_stale_flag() {",
              "  // BUG: Diagnostic contains: Cleans stale XP flags",
              "  return experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG);",
              " }",
              "}")
          .addOutputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public boolean return_contains_stale_flag() {",
              "  return true;",
              " }",
              "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void negative() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
          BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      BugCheckerRefactoringTestHelper.ExpectOutput eo =
          bcr.addInput("XPFlagCleanerNegativeCases.java");

      eo.expectUnchanged();
      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void positiveCaseWithFlagNameAsVariable() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
              BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);
      bcr.addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
              " private XPTest experimentation;",
              " public String evaluate() {",
              "  // BUG: Diagnostic contains: Cleans stale XP flags",
              "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
              "     else { return \"Y\";}",
              " }",
              "}")
              .addOutputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      " private XPTest experimentation;",
                      " public String evaluate() {",
                      "  return \"Y\";",
                      " }",
                      "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void positiveCaseWithFlagNameAsStringLiteral() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
              BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);
      bcr.addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public String evaluate() {",
              "  // BUG: Diagnostic contains: Cleans stale XP flags",
              "  if (experimentation.isToggleDisabled(\"STALE_FLAG\")) { return \"X\"; }",
              "     else { return \"Y\";}",
              " }",
              "}")
              .addOutputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      " private XPTest experimentation;",
                      " public String evaluate() {",
                      "  return \"Y\";",
                      " }",
                      "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void negativeCaseWithFlagNameAsStringLiteral() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
              BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);
      bcr.addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              " private XPTest experimentation;",
              " public String evaluate() {",
              "  if (experimentation.isToggleDisabled(\"NOT_STALE_FLAG\")) { return \"X\"; }",
              "     else { return \"Y\";}",
              " }",
              "}")
              .addOutputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      " private XPTest experimentation;",
                      " public String evaluate() {",
                      "  if (experimentation.isToggleDisabled(\"NOT_STALE_FLAG\")) { return \"X\"; }",
                      "     else { return \"Y\";}",
                      " }",
                      "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }

  @Test
  public void negativeCaseWithFlagNameAsVariable() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/piranha.properties");

    try {
      BugCheckerRefactoringTestHelper bcr =
              BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

      bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

      bcr = addHelperClasses(bcr);
      bcr.addInputLines(
              "XPFlagCleanerSinglePositiveCase.java",
              "package com.uber.piranha;",
              "class XPFlagCleanerSinglePositiveCase {",
              "private static final String STALE_FLAG_CONSTANTS = \"NOT_STALE_FLAG\";",
              " private XPTest experimentation;",
              " public String evaluate() {",
              "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
              "     else { return \"Y\";}",
              " }",
              "}")
              .addOutputLines(
                      "XPFlagCleanerSinglePositiveCase.java",
                      "package com.uber.piranha;",
                      "class XPFlagCleanerSinglePositiveCase {",
                      "private static final String STALE_FLAG_CONSTANTS = \"NOT_STALE_FLAG\";",
                      " private XPTest experimentation;",
                      " public String evaluate() {",
                      "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
                      "     else { return \"Y\";}",
                      " }",
                      "}");

      bcr.doTest();
    } catch (ParseException pe) {
      pe.printStackTrace();
      assert_().fail("Incorrect parameters passed to the checker");
    }
  }
}
