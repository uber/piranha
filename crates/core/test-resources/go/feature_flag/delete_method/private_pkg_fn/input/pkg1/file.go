/*
Copyright (c) 2023 Uber Technologies, Inc.

 <p>Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 except in compliance with the License. You may obtain a copy of the License at
 <p>http://www.apache.org/licenses/LICENSE-2.0

 <p>Unless required by applicable law or agreed to in writing, software distributed under the
 License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 express or implied. See the License for the specific language governing permissions and
 limitations under the License.
*/

package pkg1

import (
    "fmt"
)

// ProcessData processes some data using feature flags
func ProcessData(input string) string {
    if IsFeatureEnabled() {
        return fmt.Sprintf("processed: %s", input)
    }
    return input
}

// HelperFunction demonstrates package scope rule application
func HelperFunction() {
    result := someFunc()
    if result {
        fmt.Println("Feature is enabled in sub folder")
    }
}
