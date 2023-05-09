/**
 * Copyright (c) 2023 Uber Technologies, Inc.
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
package folder_2.folder_2_1;

import dagger.Provides;
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import javax.inject.Inject;

class SomeTestClass {


  enum TestEmptyEnum {}

  enum TestExperimentName {
    STALE_FLAG,
  }

  public int return_within_if_additional(int x) {
    if (x == 0) {

      if (experimentation.isToggleEnabled(TestExperimentName.SOME_OTHER_FLAG)) {
        System.out.println();
        return 0;
      }
      return 75;
    }

    if (x == 1)
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        return 1;
      } else {
        return 76;
      }

    if (x == 2) {
      int y = 3;
      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        y++;
        return y;
      }
      return y + 10;
    }

    if (x == 3) {
      int z = 4;

      if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
        z++;
      } else {
        z = z * 5;
        return z + 10;
      }
      return z;
    }

    return 100;
  }
}
