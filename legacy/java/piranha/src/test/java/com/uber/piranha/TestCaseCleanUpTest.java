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
import com.google.errorprone.ErrorProneFlags;
import java.io.IOException;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

/**
 * Test Piranha logic for remove obsolete/stale unit tests referencing the flags to be deleted (or
 * stale test annotations and value set up calls).
 */
@RunWith(JUnit4.class)
public class TestCaseCleanUpTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private BugCheckerRefactoringTestHelper addExperimentFlagEnums(
      BugCheckerRefactoringTestHelper bcr) {
    return bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG,",
            " OTHER_FLAG_1,",
            " OTHER_FLAG_2,",
            "}")
        .addOutputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " OTHER_FLAG_1,",
            " OTHER_FLAG_2,",
            "}");
  }

  private BugCheckerRefactoringTestHelper addToggleTestingAnnotationHelperClass(
      BugCheckerRefactoringTestHelper bcr) {
    return bcr.addInputLines(
            "ToggleTesting.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "@Retention(RetentionPolicy.RUNTIME)",
            "@Target({ElementType.METHOD})",
            "@interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            "}")
        .addOutputLines(
            "ToggleTesting.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "@Retention(RetentionPolicy.RUNTIME)",
            "@Target({ElementType.METHOD})",
            "@interface ToggleTesting {",
            "  TestExperimentName[] treated();",
            "}");
  }

  private BugCheckerRefactoringTestHelper addToggleTreatedAndControlAnnotationHelperClasses(
      BugCheckerRefactoringTestHelper bcr) {
    return bcr.addInputLines(
            "NewToggleTreatedSet.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Target;",
            "@Target({ElementType.METHOD})",
            "public @interface NewToggleTreatedSet {",
            "    NewToggleTreated[] value();",
            "}")
        .addOutputLines(
            "NewToggleTreatedSet.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Target;",
            "@Target({ElementType.METHOD})",
            "public @interface NewToggleTreatedSet {",
            "    NewToggleTreated[] value();",
            "}")
        .addInputLines(
            "NewToggleControlSet.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Target;",
            "@Target({ElementType.METHOD})",
            "public @interface NewToggleControlSet {",
            "    NewToggleControl[] value();",
            "}")
        .addOutputLines(
            "NewToggleControlSet.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Target;",
            "@Target({ElementType.METHOD})",
            "public @interface NewToggleControlSet {",
            "    NewToggleControl[] value();",
            "}")
        .addInputLines(
            "NewToggleTreated.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Repeatable;",
            "import java.lang.annotation.Target;",
            "@Repeatable(NewToggleTreatedSet.class)",
            "@Target({ElementType.METHOD})",
            "@interface NewToggleTreated {",
            "  String[] value();",
            "}")
        .addOutputLines(
            "NewToggleTreated.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Repeatable;",
            "import java.lang.annotation.Target;",
            "@Repeatable(NewToggleTreatedSet.class)",
            "@Target({ElementType.METHOD})",
            "@interface NewToggleTreated {",
            "  String[] value();",
            "}")
        .addInputLines(
            "NewToggleControl.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Repeatable;",
            "import java.lang.annotation.Target;",
            "@Repeatable(NewToggleControlSet.class)",
            "@Target({ElementType.METHOD})",
            "@interface NewToggleControl {",
            "  String[] value();",
            "}")
        .addOutputLines(
            "NewToggleControl.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Repeatable;",
            "import java.lang.annotation.Target;",
            "@Repeatable(NewToggleControlSet.class)",
            "@Target({ElementType.METHOD})",
            "@interface NewToggleControl {",
            "  String[] value();",
            "}");
  }

  @Test
  public void testAnnotatedTestMethodsSimpleNameImportTreated() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr = addToggleTestingAnnotationHelperClass(bcr); // Adds @ToggleTesting
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = STALE_FLAG)",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  public void x () {}",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsSimpleNameImportControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr = addToggleTestingAnnotationHelperClass(bcr); // Adds @ToggleTesting
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = STALE_FLAG)",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsArrayExpressionTreated() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr = addToggleTestingAnnotationHelperClass(bcr); // Adds @ToggleTesting
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = {OTHER_FLAG_1, STALE_FLAG})",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = {OTHER_FLAG_1})",
            "  public void x () {}",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsArrayExpressionControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr = addToggleTestingAnnotationHelperClass(bcr); // Adds @ToggleTesting
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = {OTHER_FLAG_1, STALE_FLAG})",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            // Ideally should be removed too: (GJF will do it, though :) )
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  public void foo () {}",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsClearEmptyTestAnnotation() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr = addToggleTestingAnnotationHelperClass(bcr); // Adds @ToggleTesting
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  public void foo () {}",
            "  @ToggleTesting(treated = {STALE_FLAG})",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  public void x () {}",
            "}")
        .doTest();
  }

  /**
   * [https://github.com/uber/piranha/issues/44]
   *
   * <p>This test ensures annotated methods are not removed by piranha if an empty flag is passed in
   * config
   */
  @Test
  public void testEmptyFlagDoesNotRemoveAnnotatedMethods() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr.addInputLines(
            "EmptyFlagRemovesAnnotatedMethods.java",
            "package com.uber.piranha;",
            "class EmptyFlagRemovesAnnotatedMethods {",
            "  enum TestExperimentName {",
            "    SAMPLE_STALE_FLAG",
            "  }",
            "  public String evaluate(int x) {",
            "    if (x == 1) { return \"yes\"; }",
            "    return \"no\";",
            "  }",
            "  @ToggleTesting(treated = TestExperimentName.SAMPLE_STALE_FLAG)",
            "  public void x () {}",
            "}")
        .addOutputLines(
            "EmptyFlagRemovesAnnotatedMethods.java",
            "package com.uber.piranha;",
            "class EmptyFlagRemovesAnnotatedMethods {",
            "  enum TestExperimentName {",
            "    SAMPLE_STALE_FLAG",
            "  }",
            "  public String evaluate(int x) {",
            "    if (x == 1) { return \"yes\"; }",
            "    return \"no\";",
            "  }",
            "  @ToggleTesting(treated = TestExperimentName.SAMPLE_STALE_FLAG)",
            "  public void x () {}",
            "}")
        .addInputLines(
            "ToggleTesting.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "@Retention(RetentionPolicy.RUNTIME)",
            "@Target({ElementType.METHOD})",
            "@interface ToggleTesting {",
            "  EmptyFlagRemovesAnnotatedMethods.TestExperimentName treated();",
            "}")
        .addOutputLines(
            "ToggleTesting.java",
            "package com.uber.piranha;",
            "import java.lang.annotation.ElementType;",
            "import java.lang.annotation.Retention;",
            "import java.lang.annotation.RetentionPolicy;",
            "import java.lang.annotation.Target;",
            "@Retention(RetentionPolicy.RUNTIME)",
            "@Target({ElementType.METHOD})",
            "@interface ToggleTesting {",
            "  EmptyFlagRemovesAnnotatedMethods.TestExperimentName treated();",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsWithCustomSpecTreated() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr =
        addToggleTreatedAndControlAnnotationHelperClasses(
            bcr); // Adds @NewToggleTreated and @NewToggleControl
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void bar () {}",
            "  @NewToggleTreated(\"STALE_FLAG\")",
            "  public void x () {}",
            "  @NewToggleControl(\"STALE_FLAG\")",
            "  public void y () {}",
            "  @NewToggleTreated(\"STALE_FLAG\")",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void z () {}",
            "  @NewToggleControl(\"STALE_FLAG\")",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void w () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void bar () {}",
            "  public void x () {}",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void z () {}",
            "}")
        .doTest();
  }

  @Test
  public void testAnnotatedTestMethodsWithCustomSpecControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr =
        addToggleTreatedAndControlAnnotationHelperClasses(
            bcr); // Adds @NewToggleTreated and @NewToggleControl
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  @NewToggleControl(\"OTHER_FLAG_1\")",
            "  public void bar () {}",
            "  @NewToggleTreated(\"STALE_FLAG\")",
            "  public void x () {}",
            "  @NewToggleControl(\"STALE_FLAG\")",
            "  public void y () {}",
            "  @NewToggleTreated(\"STALE_FLAG\")",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void z () {}",
            "  @NewToggleControl(\"STALE_FLAG\")",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void w () {}",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "class TestClass {",
            "  public void foo () {}",
            "  @NewToggleControl(\"OTHER_FLAG_1\")",
            "  public void bar () {}",
            "  public void y () {}",
            "  @NewToggleTreated(\"OTHER_FLAG_1\")",
            "  public void w () {}",
            "}")
        .doTest();
  }

  @Test
  public void testCleanBySettersHeuristicMinimal() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     System.err.println(\"To be removed\");",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "}")
        .doTest();
  }

  @Test
  public void testCleanBySettersHeuristicControl() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     System.err.println(\"To be removed\");",
            "  }",
            "  @Test",
            "  public void test_StaleFlag_control() {",
            "     experimentation.putToggleDisabled(STALE_FLAG);",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_BothFlags_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Call removed\");",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_control() {",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_BothFlags_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Call removed\");",
            "  }",
            "}")
        .doTest();
  }

  @Test
  public void testCleanBySettersHeuristicTreatment() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_StaleFlag_control() {",
            "     experimentation.putToggleDisabled(STALE_FLAG);",
            "     System.err.println(\"To be removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_BothFlags_control() {",
            "     experimentation.putToggleDisabled(STALE_FLAG);",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Call removed\");",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_BothFlags_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Call removed\");",
            "  }",
            "}")
        .doTest();
  }

  @Test
  public void testCleanBySettersHeuristicIgnoreNested() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag("Piranha:Config", "config/properties.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated(int x) {",
            "     if (x == 1) {",
            "       experimentation.putToggleEnabled(STALE_FLAG);",
            "       System.err.println(\"Call removed\");",
            "     }",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated(int x) {",
            "     if (x == 1) {",
            "       System.err.println(\"Call removed\");",
            "     }",
            "  }",
            "}")
        .doTest();
  }

  @Test
  public void testCleanBySettersHeuristicIgnoreSettersForOtherFlags() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "false");
    b.putFlag(
        "Piranha:Config",
        "src/test/resources/config/properties_test_clean_by_setters_ignore_others.json");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new XPFlagCleaner(b.build()), getClass());

    bcr = bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    bcr = PiranhaTestingHelpers.addHelperClasses(bcr);
    bcr = addExperimentFlagEnums(bcr); // Adds STALE_FLAG, etc enums
    bcr.addInputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.STALE_FLAG;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     System.err.println(\"To be removed\");",
            "  }",
            "  @Test",
            "  public void test_StaleFlag_control() {",
            "     experimentation.putToggleDisabled(STALE_FLAG);",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_BothFlags_treated() {",
            "     experimentation.putToggleEnabled(STALE_FLAG);",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Call removed\");",
            "  }",
            "}")
        .addOutputLines(
            "TestClass.java",
            "package com.uber.piranha;",
            "import org.junit.Test;",
            "import static com.uber.piranha.TestExperimentName.OTHER_FLAG_1;",
            "class TestClass {",
            "  private XPTest experimentation;",
            "  @Test",
            "  public void test_StaleFlag_control() {",
            "     System.err.println(\"Call above removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_treated() {",
            "     experimentation.putToggleEnabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "  @Test",
            "  public void test_OtherFlag_control() {",
            "     experimentation.putToggleDisabled(OTHER_FLAG_1);",
            "     System.err.println(\"Not removed\");",
            "  }",
            "}")
        .doTest();
  }
}
