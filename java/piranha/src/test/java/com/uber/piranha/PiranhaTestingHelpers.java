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
import java.io.IOException;

/** Static helper methods used across the various Piranha test suite classes */
public final class PiranhaTestingHelpers {

  public static BugCheckerRefactoringTestHelper addHelperClasses(
      BugCheckerRefactoringTestHelper bcr) throws IOException {
    return bcr.addInput("XPTest.java").expectUnchanged();
  }

  public static BugCheckerRefactoringTestHelper addExperimentFlagEnumsWithConstructor(
      BugCheckerRefactoringTestHelper bcr, boolean hasEnumConfiguration) {
    BugCheckerRefactoringTestHelper.ExpectOutput inputLines =
        bcr.addInputLines(
            "TestExperimentName.java",
            "package com.uber.piranha;",
            "public enum TestExperimentName {",
            " STALE_FLAG(\"stale.flag\"),",
            " OTHER_FLAG(\"other\");",
            " private final String key;",
            " public String getKey() {",
            "   return key;",
            " }",
            " TestExperimentName(final String key) {",
            "  this.key = key;",
            " }",
            "}");
    return hasEnumConfiguration
        ? inputLines.addOutputLines(
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
        : inputLines.expectUnchanged();
  }
}
