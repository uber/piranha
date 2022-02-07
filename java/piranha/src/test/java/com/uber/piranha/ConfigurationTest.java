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

import static com.uber.piranha.PiranhaTestingHelpers.addExperimentFlagEnumsWithConstructor;
import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.containsString;

import com.google.errorprone.BugCheckerRefactoringTestHelper;
import com.google.errorprone.CompilationTestHelper;
import com.google.errorprone.ErrorProneFlags;
import com.uber.piranha.config.PiranhaConfigurationException;
import java.io.IOException;
import java.util.Arrays;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.ExpectedException;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

/**
 * Test cases related to Piranha configuration. Particularly parsing and handling of the
 * properties.json configuration file.
 */
@RunWith(JUnit4.class)
public class ConfigurationTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();
  /** Used to test exception handling in init method. */
  @Rule public ExpectedException expectedEx = ExpectedException.none();

  @Test
  public void runPiranhaWithFewProperties() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_noTreatmentGroup.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath())
        .addInput("XPFlagCleanerPositiveCases.java")
        .addOutput("XPFlagCleanerPositiveCasesTreatment.java")
        .doTest();
  }

  /*
   * Test for the case when xpFlagName is ""
   * Methods with no arguments are simplified, provided argumentIndex is not specified
   * (assuming that returnType and receiverType - if specified - are a match)
   * Uses "properties_test_noFlag.json" as the config file.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void noFlagCase() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_noFlag.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isUnrelatedToggleEnabled()) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled()) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "  int a = 1;",
            "  if (experimentation.isUnrelatedToggleEnabled()) { int c = 1; }",
            "     else { int d = 2;}",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * This test passes flag name as a Piranha argument.
   * Whether or not flag name is passed, the noFlagCase simplification should not be affected.
   * Uses "properties_test_noFlag.json" as the config file.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void noFlagCaseWithFlagSpecified() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_noFlag.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isUnrelatedToggleEnabled()) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled()) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "     int a = 1;",
            "  if (experimentation.isUnrelatedToggleEnabled()) { int c = 1; }",
            "     else { int d = 2;}",
            "     return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void noFlagCaseWithReturnType() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleDisabled()) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void noFlagCaseWithReceiverType() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleDisabled()) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled()) { int a = 1; }",
            "     else { int b = 2;}",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void stringLiteralFlagWithReturn() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (\"true\".equals(experimentation.flagMethodThatReturnsStringObject())) { int e = 1; }",
            "     else { int f = 2;}",
            "  if (experimentation.flagMethodThatReturnsBooleanObject()) { int g = 1; }",
            "     else { int h = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int c = 1;",
            "  if (\"true\".equals(true)) { int e = 1; }",
            "     else { int f = 2;}",
            "  int g = 1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void variableFlagWithReturn() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (\"true\".equals(experimentation.flagMethodThatReturnsStringObject())) { int e = 1; }",
            "     else { int f = 2;}",
            "  if (experimentation.flagMethodThatReturnsBooleanObject()) { int g = 1; }",
            "     else { int h = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int c = 1;",
            "  if (\"true\".equals(true)) { int e = 1; }",
            "     else { int f = 2;}",
            "  int g = 1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void stringLiteralFlagWithReceiver() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int c = 1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void variableFlagWithReceiver() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int c = 1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent argumentIndex specified.
   * Hence, "isToggleEnabled" is not simplified.
   * "isToggleEnabledWithMultipleArguments" has two occurrences, only one matches the specified argumentIndex.
   * */
  @Test
  public void variableFlagWithArgumentIndex() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c = 1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent argumentIndex specified.
   * A different "Piranha:FlagName" argument is specified,
   * hence no simplification occurs for method properties with argumentIndex specified.
   * */
  @Test
  public void variableFlagWithArgumentIndexWhenWrongFlag() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_argument.json");

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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " private XPTest experimentation;",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent argumentIndex specified.
   * Hence, "isToggleEnabled" is not simplified.
   * "isToggleEnabledWithMultipleArguments" has two occurrences, only one matches the specified argumentIndex.
   * */
  @Test
  public void stringLiteralFlagWithArgumentIndex() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent argumentIndex specified.
   * A different "Piranha:FlagName" argument is specified,
   * hence no simplification occurs for method properties with argumentIndex specified.
   * */
  @Test
  public void stringLiteralFlagWithArgumentIndexWhenWrongFlag() throws IOException {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(\"STALE_FLAG\")) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * "isToggleEnabledWithMultipleArguments" has two occurrences, only one matches the specified argumentIndex.
   * */
  @Test
  public void stringLiteralFlagWithReceiverAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType + argumentIndex combination specified.
   * A different "Piranha:FlagName" argument is specified, hence no simplification occurs.
   * */
  @Test
  public void stringLiteralFlagWithReceiverAndArgumentWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(\"STALE_FLAG\")) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * "isToggleEnabledWithMultipleArguments" has two occurrences, only one matches the specified argumentIndex.
   * */
  @Test
  public void stringLiteralFlagWithReturnAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * "isToggleEnabledWithMultipleArguments" has two occurrences, only one matches the specified argumentIndex.
   * */
  @Test
  public void variableFlagWithReturnAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + argumentIndex combination specified.
   * A different "Piranha:FlagName" argument is specified, hence no simplification occurs.
   * */
  @Test
  public void variableFlagWithReturnAndArgumentWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * */
  @Test
  public void variableFlagWithReceiverAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_receive_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent receiverType + argumentIndex combination specified.
   * A different "Piranha:FlagName" argument is specified, hence no simplification occurs.
   * */
  @Test
  public void variableFlagWithReceiverAndArgumentWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void variableFlagWithReturnAndReceiver() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  int i = 1;",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * A different "Piranha:FlagName" argument is specified, but "Piranha:ArgumentIndexOptional" is true and
   * no argumentIndex is specified in the config file, hence simplification is carried out irrespective of flag name.
   * */
  @Test
  public void variableFlagWithReturnAndReceiverWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(STALE_FLAG_CONSTANTS)) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .addOutputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  int i = 1;",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + argumentIndex combination specified.
   * A different "Piranha:FlagName" argument is specified, hence no simplification occurs.
   * */
  @Test
  public void stringLiteralFlagWithReturnAndArgumentWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
            "  if (experimentation.isToggleDisabled(\"STALE_FLAG\")) { return \"X\"; }",
            "     else { return \"Y\";}",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * Note that "Piranha:FlagName" is not considered when "Piranha:ArgumentIndexOptional" is set to true and argumentIndex is not specified.
   * */
  @Test
  public void stringLiteralFlagWithReturnAndReceiver() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  int i = 1;",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * A different "Piranha:FlagName" argument is specified, but "Piranha:ArgumentIndexOptional" is true and
   * no argumentIndex is specified in the config file, hence simplification is carried out irrespective of flag name.
   * */
  @Test
  public void stringLiteralFlagWithReturnAndReceiverWhenWrongFlag() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "NOT_STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_return_receive.json");
    b.putFlag("Piranha:ArgumentIndexOptional", "true");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  int i = 1;",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * */
  @Test
  public void stringLiteralFlagWithReturnAndReceiverAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag(
        "Piranha:Config", "src/test/resources/config/properties_test_return_receive_argument.json");

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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, \"STALE_FLAG\")) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(\"STALE_FLAG\")) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(\"STALE_FLAG\")) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(\"STALE_FLAG\", 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_return_receive_argument.json" instead of "properties.json".
   * In it, the method "isToggleEnabled" has a non-existent returnType + receiverType + argumentIndex combination specified.
   * Hence, "isToggleEnabled" is not simplified.
   * */
  @Test
  public void variableFlagWithReturnAndReceiverAndArgument() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag(
        "Piranha:Config", "src/test/resources/config/properties_test_return_receive_argument.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    // can be removed after Piranha implements two pass analysis
    bcr = bcr.allowBreakingChanges();

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "XPFlagCleanerSinglePositiveCase.java",
            "package com.uber.piranha;",
            "class XPFlagCleanerSinglePositiveCase {",
            " private XPTest experimentation;",
            "private static final String STALE_FLAG_CONSTANTS = \"STALE_FLAG\";",
            " public String evaluate() {",
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(123, STALE_FLAG_CONSTANTS)) { int g = 1; }",
            "     else { int h = 2;}",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  if (experimentation.isFlagTreated(STALE_FLAG_CONSTANTS)) { int c = 1; }",
            "     else { int d = 2;}",
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
            "  if (experimentation.isToggleEnabled(STALE_FLAG_CONSTANTS)) { int a = 1; }",
            "     else { int b = 2;}",
            "  int g = 1;",
            "  if (experimentation.isToggleEnabledWithMultipleArguments(STALE_FLAG_CONSTANTS, 123)) { int i = 1; }",
            "     else { int j = 2;}",
            "  int c =1;",
            "  return \"Y\";",
            " }",
            "}")
        .doTest();
  }

  /*
   * Uses "properties_test_invalid.json" instead of "properties.json".
   * When "methodProperties" is not specified,
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenConfigFileInvalid() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/properties_test_invalid.json");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("methodProperties not found, required.");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * Check that the user is warned when using an old style piranha.properties file
   * */
  @Test
  public void warningWhenConfigFileOld() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/piranha.properties");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage(
        "WARNING: With version 0.1.0, PiranhaJava has changed its configuration file format");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * When "Piranha:Config" Piranha argument is passed an empty string,
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenConfigPathNotSpecified() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("Provided config file not found");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * When "Piranha:Config" Piranha argument is passed an invalid path,
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenInvalidPathSpecified() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "this/path/does/not/exist");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("Provided config file not found");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * Uses "properties_test_invalid_2.json" as the config file.
   * When any method property does not have "methodName",
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenMethodNameNotSpecified() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/properties_test_invalid_2.json");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("methodProperty is missing mandatory methodName field");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * Uses "properties_test_invalid_3.json" as the config file.
   * When any method property does not have "flagType",
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenFlagTypeNotSpecified() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/properties_test_invalid_3.json");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("methodProperty is missing mandatory flagType field");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /*
   * Uses "properties_test_invalid_4.json" as the config file.
   * When any method property does not have "argumentIndex", and "Piranha:ArgumentIndexOptional" is not set to true,
   * raise PiranhaConfigurationException with appropriate exception message.
   * */
  @Test
  public void exceptionWhenArgumentIndexNotSpecifiedAndNotArgumentIndexOptional() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/properties_test_invalid_4.json");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("methodProperty did not have argumentIndex");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /**
   * Uses "properties_test_without_enum.json" instead of "properties.json". In it, there is no
   * "enumProperties" specified. Thus, we do not remove the enum constant with the matching
   * "stale.flag" constructor, STALE_FLAG.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/141">Issue #141</a>
   */
  @Test
  public void testConfigWithoutRemoveEnumMatch() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "src/test/resources/config/properties_test_without_enum.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, false); // Adds STALE_FLAG, etc enums

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr.addInputLines(
            "XPFlagCleanerRemoveMatchingEnum.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class XPFlagCleanerRemoveMatchingEnum {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG.getKey());",
            "  }",
            "}")
        .expectUnchanged()
        .doTest();
    ;
  }

  /**
   * Uses "properties_test_with_enum_no_match.json" instead of "properties.json". In it, there is an
   * "enumProperties" specified, but it is for a different enum class than tested here. Thus, we do
   * not remove the enum constant with the matching "stale.flag" constructor, STALE_FLAG.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/141">Issue #141</a>
   */
  @Test
  public void testConfigWithEnumNoMatch() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag(
        "Piranha:Config", "src/test/resources/config/properties_test_with_enum_no_match.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, false); // Adds STALE_FLAG, etc enums

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);

    bcr.addInputLines(
            "XPFlagCleanerRemoveMatchingEnum.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class XPFlagCleanerRemoveMatchingEnum {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG.getKey());",
            "  }",
            "}")
        .expectUnchanged()
        .doTest();
    ;
  }

  /**
   * Uses "properties_test_with_enum_wrong_arg.json" instead of "properties.json". In it, there is
   * an "enumProperties" specified, but it is for the wrong constructor argument index. Thus, we do
   * not remove the enum constant with the matching "stale.flag" constructor, STALE_FLAG.
   *
   * <p>See <a href="https://github.com/uber/piranha/issues/141">Issue #141</a>
   */
  @Test
  public void testConfigWithEnumWrongArg() throws Exception {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "stale.flag");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag(
        "Piranha:Config", "src/test/resources/config/properties_test_with_enum_wrong_arg.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = addExperimentFlagEnumsWithConstructor(bcr, false); // Adds STALE_FLAG, etc enums

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);

    bcr.addInputLines(
            "XPFlagCleanerRemoveMatchingEnum.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class XPFlagCleanerRemoveMatchingEnum {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG.getKey());",
            "  }",
            "}")
        .expectUnchanged()
        .doTest();
    ;
  }

  /**
   * This test ensures 'PiranhaConfigurationException' is thrown if the 'Piranha:FlagName' is
   * missing/not-provided in the config
   */
  @Test
  public void testPiranhaConfigErrorWhenFlagNotProvided() throws Exception {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage("Piranha:FlagName is missing");

    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }

  /**
   * This test ensures 'PiranhaConfigurationException' is propagated as en error if the
   * configuration file is missing
   */
  @Test
  public void testPiranhaCrashOnNoConfig() {
    // Error Prone turns Runtime Exceptions inside EP into blocks of text (including the exception
    // trace) inside a new AssertionError exception. This is actually looking for
    // PiranhaConfigurationException inside XPFlagCleaner.
    expectedEx.expect(AssertionError.class);
    expectedEx.expectMessage(
        "An unhandled exception was thrown by the Error Prone static analysis plugin");
    expectedEx.expectMessage("PiranhaConfigurationException: Error reading config file");
    expectedEx.expectMessage("java.io.IOException: Provided config file not found");

    CompilationTestHelper compilationHelper =
        CompilationTestHelper.newInstance(XPFlagCleaner.class, getClass());
    compilationHelper
        .setArgs(
            Arrays.asList(
                "-d",
                temporaryFolder.getRoot().getAbsolutePath(),
                "-XepOpt:Piranha:FlagName=noop",
                "-XepOpt:Piranha:IsTreated=true",
                "-XepOpt:Piranha:Config=src/test/resources/config/nonexistent_file"))
        .addSourceLines("Dummy.java", "package com.uber.piranha;", "class Dummy {", "}")
        .doTest();
  }

  /**
   * This test ensures 'PiranhaConfigurationException' is propagated as en error if the
   * configuration file is using the outdated .properties format
   */
  @Test
  public void testPiranhaCrashOnOldConfig() {
    // Error Prone turns Runtime Exceptions inside EP into blocks of text (including the exception
    // trace) inside a new AssertionError exception. This is actually looking for
    // PiranhaConfigurationException inside XPFlagCleaner.
    expectedEx.expect(AssertionError.class);
    expectedEx.expectMessage(
        "An unhandled exception was thrown by the Error Prone static analysis plugin");
    expectedEx.expectMessage(
        "PiranhaConfigurationException: Invalid or incorrectly formatted config file. ");
    expectedEx.expectMessage(
        "WARNING: With version 0.1.0, PiranhaJava has changed its configuration file format to json");

    CompilationTestHelper compilationHelper =
        CompilationTestHelper.newInstance(XPFlagCleaner.class, getClass());
    compilationHelper
        .setArgs(
            Arrays.asList(
                "-d",
                temporaryFolder.getRoot().getAbsolutePath(),
                "-XepOpt:Piranha:FlagName=noop",
                "-XepOpt:Piranha:IsTreated=true",
                "-XepOpt:Piranha:Config=src/test/resources/config/invalid/piranha.properties"))
        .addSourceLines("Dummy.java", "package com.uber.piranha;", "class Dummy {", "}")
        .doTest();
  }

  @Test
  public void test_wrongConfig() {
    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "src/test/resources/config/invalid/propetiesDoNotExist.json");
    expectedEx.expect(PiranhaConfigurationException.class);
    expectedEx.expectMessage(
        allOf(
            containsString("Error reading config file"),
            containsString("Provided config file not found")));
    XPFlagCleaner flagCleaner = new XPFlagCleaner();
    flagCleaner.init(b.build());
  }
}
