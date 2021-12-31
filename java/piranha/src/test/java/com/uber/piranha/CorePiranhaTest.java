/**
 * Copyright (c) 2019-2021 Uber Technologies, Inc.
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

import com.google.errorprone.BugCheckerRefactoringTestHelper;
import com.google.errorprone.CompilationTestHelper;
import com.google.errorprone.ErrorProneFlags;
import java.io.IOException;
import java.util.Arrays;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

/**
 * This test suite tests core Piranha logic not fitting in any of the other test suites.
 *
 * <p>Additionally, we run the tests in resources/... from here.
 */
@RunWith(JUnit4.class)
public class CorePiranhaTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private CompilationTestHelper compilationHelper;

  @Before
  public void setup() {
    compilationHelper =
        CompilationTestHelper.newInstance(XPFlagCleaner.class, getClass())
            .setArgs(Arrays.asList("-d", temporaryFolder.getRoot().getAbsolutePath()));
  }

  @Test
  public void test_xpflagsPositiveCases() {
    compilationHelper
        .setArgs(
            Arrays.asList(
                "-d",
                temporaryFolder.getRoot().getAbsolutePath(),
                "-XepOpt:Piranha:FlagName=STALE_FLAG",
                "-XepOpt:Piranha:IsTreated=true",
                "-XepOpt:Piranha:Config=config/properties.json"))
        .addSourceFile("XPFlagCleanerPositiveCases.java")
        .doTest();
  }

  @Test
  public void test_xpflagsNegativeCases() {
    compilationHelper
        .setArgs(
            Arrays.asList(
                "-d",
                temporaryFolder.getRoot().getAbsolutePath(),
                "-XepOpt:Piranha:FlagName=STALE_FLAG",
                "-XepOpt:Piranha:IsTreated=true",
                "-XepOpt:Piranha:Config=config/properties.json"))
        .addSourceFile("XPFlagCleanerNegativeCases.java")
        .doTest();
  }

  @Test
  public void positiveTreatment() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath())
        .addInput("XPFlagCleanerPositiveCases.java")
        .addOutput("XPFlagCleanerPositiveCasesTreatment.java")
        .doTest();
  }

  @Test
  public void positiveControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath())
        .addInput("XPFlagCleanerPositiveCases.java")
        .addOutput("XPFlagCleanerPositiveCasesControl.java")
        .doTest();
  }

  @Test
  public void positiveRemoveImport() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);

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
            "import static com.uber.piranha.TestExperimentName" + ".STALE_FLAG;",
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
            "}")
        .doTest();
  }

  @Test
  public void negative() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath())
        .addInput("XPFlagCleanerNegativeCases.java")
        .expectUnchanged()
        .doTest();
  }

  @Test
  public void positiveCaseWithFlagNameAsVariable() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
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
            "}")
        .doTest();
  }

  @Test
  public void positiveCaseWithFlagNameAsStringLiteral() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
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
            "}")
        .doTest();
  }

  @Test
  public void negativeCaseWithFlagNameAsStringLiteral() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
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
            "}")
        .doTest();
  }

  @Test
  public void negativeCaseWithFlagNameAsVariable() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
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
            "}")
        .doTest();
  }

  /**
   * [https://github.com/uber/piranha/issues/44]
   *
   * <p>This test ensures static imports are not removed by piranha if an empty flag is passed in
   * config
   */
  @Test
  public void testEmptyFlagDoesNotRemoveStaticImports() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "EmptyFlagRemovesStaticImports.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.Constants.ONE;",
            "class EmptyFlagRemovesStaticImports {",
            "  public String evaluate(int x) {",
            "    if (x == ONE) { return \"yes\"; }",
            "    return \"no\";",
            "  }",
            "}")
        .addOutputLines(
            "EmptyFlagRemovesStaticImports.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.Constants.ONE;",
            "class EmptyFlagRemovesStaticImports {",
            "  public String evaluate(int x) {",
            "    if (x == ONE) { return \"yes\"; }",
            "    return \"no\";",
            "  }",
            "}")
        .addInputLines(
            "Constants.java",
            "package com.uber.piranha;",
            "class Constants {",
            "  public static int ONE = 1;",
            "}")
        .addOutputLines(
            "Constants.java",
            "package com.uber.piranha;",
            "class Constants {",
            "  public static int ONE = 1;",
            "}")
        .doTest();
  }

  @Test
  public void testMultipleClonesAcrossFiles() throws IOException {
    // This test mostly ensures we aren't persisting state (such as overlap check state) between
    // compilation units in a way that will fail to clean similar code across multiple files.

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);

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
        // Order is back to front: the following refactoring will fail before this commit (clearing
        // endPos)...
        .addInputLines(
            "XPFlagCleanerSinglePositiveCase1.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class XPFlagCleanerSinglePositiveCase1 {",
            " private XPTest experimentation;",
            " public void foo() {",
            "  // BUG: Diagnostic contains: Cleans stale XP flags",
            "  if (experimentation.isToggleDisabled(STALE_FLAG)) {",
            "     System.err.println(\"To be removed\");",
            "  }",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase1.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase1 {",
            " private XPTest experimentation;",
            " public void foo() {",
            " }",
            "}")
        // ... while this identical refactoring succeeds:
        .addInputLines(
            "XPFlagCleanerSinglePositiveCase2.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class XPFlagCleanerSinglePositiveCase2 {",
            " private XPTest experimentation;",
            " public void foo() {",
            "  // BUG: Diagnostic contains: Cleans stale XP flags",
            "  if (experimentation.isToggleDisabled(STALE_FLAG)) {",
            "     System.err.println(\"To be removed\");",
            "  }",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase2.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase2 {",
            " private XPTest experimentation;",
            " public void foo() {",
            " }",
            "}")
        .doTest();
  }

  @Test
  public void testIgnoresPrefixMatchFlag() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG,",
            " OTHER_STALE_FLAG",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_STALE_FLAG",
            "}")
        .addInputLines(
            "TestClassFullMatch.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClassFullMatch { }")
        .addOutputLines(
            "TestClassFullMatch.java", "package com.uber.piranha;", "class TestClassFullMatch { }")
        .addInputLines(
            "TestClassPartialMatch.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_STALE_FLAG;",
            "class TestClassPartialMatch { }")
        .addOutputLines(
            "TestClassPartialMatch.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_STALE_FLAG;",
            "class TestClassPartialMatch { }")
        .doTest();
  }

  @Test
  public void testRemoveSpecificAPIpatternsMockito() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());
    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.mockito.Mockito.when;",
            "import org.mockito.invocation.InvocationOnMock;",
            "import org.mockito.stubbing.Answer;",
            "import org.mockito.stubbing.OngoingStubbing;",
            "class MockitoTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "  Answer ans = new Answer<Integer>() {\n"
                + "      public Integer answer(InvocationOnMock invocation) throws Throwable {\n"
                + "        return (Integer) invocation.getArguments()[0];\n"
                + "      }};",
            "  when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"STALE_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"STALE_FLAG\")).then(ans);",
            "  boolean b1 = someWrapper(when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenCallRealMethod());",
            "  boolean b2 = someWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod());",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"STALE_FLAG\"))).thenCallRealMethod();",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\"))).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).then(ans);",
            "  when(foobar()).thenReturn(false);",
            "  when(foobar()).thenAnswer(ans);",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            " public OngoingStubbing<Boolean> someWhenWrapper(OngoingStubbing<Boolean> x) { return x;}",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.mockito.Mockito.when;",
            "import org.mockito.invocation.InvocationOnMock;",
            "import org.mockito.stubbing.Answer;",
            "import org.mockito.stubbing.OngoingStubbing;",
            "class MockitoTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "  Answer ans = new Answer<Integer>() {\n"
                + "      public Integer answer(InvocationOnMock invocation) throws Throwable {\n"
                + "        return (Integer) invocation.getArguments()[0];\n"
                + "      }};",
            "",
            "",
            "",
            "",
            "",
            "  boolean b1 = someWrapper(when(true).thenCallRealMethod());",
            "  boolean b2 = someWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod());",
            "",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\"))).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).then(ans);",
            "  when(foobar()).thenReturn(false);",
            "  when(foobar()).thenAnswer(ans);",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            " public OngoingStubbing<Boolean> someWhenWrapper(OngoingStubbing<Boolean> x) { return x;}",
            "}")
        .doTest();
  }

  @Test
  public void testRemoveSpecificAPIpatternsEasyMock() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());
    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import org.easymock.EasyMock;",
            "class EasyMockTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).asStub();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(foobar()).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import org.easymock.EasyMock;",
            "class EasyMockTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "",
            "",
            "",
            "",
            "",
            "",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(foobar()).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            "}")
        .doTest();
  }

  @Test
  public void testRemoveSpecificAPIpatternsJUnit() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());
    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.junit.Assert.assertFalse;",
            "import static org.junit.Assert.assertTrue;",
            "class JUnitTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "  assertFalse(experimentation.isToggleDisabled(\"STALE_FLAG\"));",
            "  assertTrue(experimentation.isToggleEnabled(\"STALE_FLAG\"));",
            "  assertFalse(experimentation.isToggleEnabled(\"OTHER_FLAG\"));",
            "  assertTrue(experimentation.isToggleDisabled(\"OTHER_FLAG\"));",
            "  assertTrue(foobar());",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.junit.Assert.assertFalse;",
            "import static org.junit.Assert.assertTrue;",
            "class JUnitTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "",
            "",
            "  assertFalse(experimentation.isToggleEnabled(\"OTHER_FLAG\"));",
            "  assertTrue(experimentation.isToggleDisabled(\"OTHER_FLAG\"));",
            "  assertTrue(foobar());",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            "}")
        .doTest();
  }

  @Test
  public void testRemoveSpecificAPIpatterns() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());
    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.mockito.Mockito.when;",
            "import org.mockito.invocation.InvocationOnMock;",
            "import org.mockito.stubbing.Answer;",
            "import org.mockito.stubbing.OngoingStubbing;",
            "import org.easymock.EasyMock;",
            "import static org.junit.Assert.assertFalse;",
            "import static org.junit.Assert.assertTrue;",
            "class MockitoTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "// Mockito Test Scenarios",
            "  Answer ans = new Answer<Integer>() {\n"
                + "      public Integer answer(InvocationOnMock invocation) throws Throwable {\n"
                + "        return (Integer) invocation.getArguments()[0];\n"
                + "      }};",
            "  when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"STALE_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"STALE_FLAG\")).then(ans);",
            "  boolean b1 = someWrapper(when(experimentation.isToggleDisabled(\"STALE_FLAG\")).thenCallRealMethod());",
            "  boolean b2 = someWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod());",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"STALE_FLAG\"))).thenCallRealMethod();",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\"))).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).then(ans);",
            "  when(foobar()).thenReturn(false);",
            "  when(foobar()).thenAnswer(ans);",
            "// Easymock Test scenarios",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"STALE_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).andReturn(true);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"STALE_FLAG\")).asStub();",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(foobar()).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "// JUnit Assert Test scearios",
            "  assertFalse(experimentation.isToggleDisabled(\"STALE_FLAG\"));",
            "  assertTrue(experimentation.isToggleEnabled(\"STALE_FLAG\"));",
            "  assertFalse(experimentation.isToggleEnabled(\"OTHER_FLAG\"));",
            "  assertTrue(experimentation.isToggleDisabled(\"OTHER_FLAG\"));",
            "  assertTrue(foobar());",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            " public OngoingStubbing<Boolean> someWhenWrapper(OngoingStubbing<Boolean> x) { return x;}",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "import static org.mockito.Mockito.when;",
            "import org.mockito.invocation.InvocationOnMock;",
            "import org.mockito.stubbing.Answer;",
            "import org.mockito.stubbing.OngoingStubbing;",
            "import org.easymock.EasyMock;",
            "import static org.junit.Assert.assertFalse;",
            "import static org.junit.Assert.assertTrue;",
            "class MockitoTest {",
            " private XPTest experimentation;",
            " public void evaluate() {",
            "// Mockito Test Scenarios",
            "  Answer ans = new Answer<Integer>() {\n"
                + "      public Integer answer(InvocationOnMock invocation) throws Throwable {\n"
                + "        return (Integer) invocation.getArguments()[0];\n"
                + "      }};",
            "",
            "",
            "",
            "",
            "",
            "  boolean b1 = someWrapper(when(true).thenCallRealMethod());",
            "  boolean b2 = someWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod());",
            "",
            "  someWhenWrapper(when(experimentation.isToggleDisabled(\"OTHER_FLAG\"))).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenReturn(false);",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).thenThrow(new RuntimeException());",
            "  when(experimentation.isToggleDisabled(\"OTHER_FLAG\")).thenCallRealMethod();",
            "  when(experimentation.isToggleEnabled(\"OTHER_FLAG\")).then(ans);",
            "  when(foobar()).thenReturn(false);",
            "  when(foobar()).thenAnswer(ans);",
            "// Easymock Test scenarios",
            "",
            "",
            "",
            "",
            "",
            "",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "  EasyMock.expect(foobar()).andReturn(true).times(5);",
            "  EasyMock.expect(experimentation.isToggleEnabled(\"OTHER_FLAG\")).andReturn(true).times(5, 6);",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).once();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).atLeastOnce();",
            "  EasyMock.expect(experimentation.isToggleDisabled(\"OTHER_FLAG\")).andReturn(true).anyTimes();",
            "// JUnit Assert Test scearios",
            "",
            "",
            "  assertFalse(experimentation.isToggleEnabled(\"OTHER_FLAG\"));",
            "  assertTrue(experimentation.isToggleDisabled(\"OTHER_FLAG\"));",
            "  assertTrue(foobar());",
            " }",
            " public boolean foobar() { return true;}",
            " public boolean someWrapper(Object obj) { return true;}",
            " public OngoingStubbing<Boolean> someWhenWrapper(OngoingStubbing<Boolean> x) { return x;}",
            "}")
        .doTest();
  }
}
