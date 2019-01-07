package com.uber.piranha;

import com.google.errorprone.CompilationTestHelper;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;
import static org.junit.Assert.fail;
import java.net.URL;

import java.text.ParseException;
import java.util.Arrays;
import java.io.IOException;

import com.google.errorprone.BugCheckerRefactoringTestHelper;
import com.google.errorprone.ErrorProneFlags;

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
      fail("Incorrect parameters passed to the checker");
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
      fail("Incorrect parameters passed to the checker");
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
      fail("Incorrect parameters passed to the checker");
    }
  }
}
