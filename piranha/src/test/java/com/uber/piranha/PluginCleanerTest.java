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

@RunWith(JUnit4.class)
public class PluginCleanerTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private CompilationTestHelper compilationHelper;

  @Before
  public void setup() {
    compilationHelper = CompilationTestHelper.newInstance(PluginCleaner.class, getClass());
    compilationHelper.setArgs(Arrays.asList("-d", temporaryFolder.getRoot().getAbsolutePath()));
  }

  @Test
  public void test_plugin_cleaning() {
    compilationHelper.setArgs(
        Arrays.asList(
            "-d",
            temporaryFolder.getRoot().getAbsolutePath(),
            "-XepOpt:PluginCleaner:PluginName=StalePluginVar",
            "-XepOpt:PluginCleaner:PluginFactory=StalePluginFactory"));
    compilationHelper.addSourceFile("PluginCleanerPositiveCases.java").doTest();
  }

  @Test
  public void plugin_cleanup_io_check() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("PluginCleaner:PluginFactory", "StalePluginFactory");
    b.putFlag("PluginCleaner:PluginName", "StalePluginVar");

    BugCheckerRefactoringTestHelper bcr =
        BugCheckerRefactoringTestHelper.newInstance(new PluginCleaner(b.build()), getClass());

    bcr.setArgs("-d", temporaryFolder.getRoot().getAbsolutePath());

    BugCheckerRefactoringTestHelper.ExpectOutput eo =
        bcr.addInput("PluginCleanerPositiveCases.java");

    eo.addOutput("PluginCleanerPositiveCasesOutput.java");
    bcr.doTest();
  }
}
