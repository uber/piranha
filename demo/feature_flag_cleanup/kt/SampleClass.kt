// Copyright (c) 2023 Uber Technologies, Inc.
// 
// <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// <p>http://www.apache.org/licenses/LICENSE-2.0
// 
// <p>Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

class SampleKotlin {

     fun print_status_enabled() {
        if (experimentation.isToggleEnabled(SAMPLE_STALE_FLAG)) {
            println("SAMPLE_STALE_FLAG is enabled")
        } else {
            println("SAMPLE_STALE_FLAG is disabled")
        }
    }

    fun print_status_disabled() {
        if (experimentation.isToggleDisabled(SAMPLE_STALE_FLAG)) {
            println("SAMPLE_STALE_FLAG is disabled")
        } else {
            println("SAMPLE_STALE_FLAG is enabled")
        }
    }
}
