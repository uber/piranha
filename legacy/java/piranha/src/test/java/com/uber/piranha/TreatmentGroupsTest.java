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
 * Test Piranha logic specific to handling treatment group information (when not used as part of
 * test code removal).
 */
@RunWith(JUnit4.class)
public class TreatmentGroupsTest {
  @Rule public TemporaryFolder temporaryFolder = new TemporaryFolder();

  @Test
  public void positiveSpecificTreatmentGroup() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:TreatmentGroup", "GROUP_A");
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
            "}")
        .doTest();
  }

  @Test
  public void dontRemoveGenericallyNamedTreatmentGroups() throws IOException {

    ErrorProneFlags.Builder b = ErrorProneFlags.builder();
    b.putFlag("Piranha:FlagName", "STALE_FLAG");
    b.putFlag("Piranha:IsTreated", "true");
    b.putFlag("Piranha:TreatmentGroup", "TREATED");
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
            "}")
        .doTest();
  }
}
