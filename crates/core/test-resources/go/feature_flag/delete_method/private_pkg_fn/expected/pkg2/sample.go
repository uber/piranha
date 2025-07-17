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

package main

import (
    "fmt"
    "mycompany/featureflags/exp"
)

func someFunc() bool {  
    // Simulate a real feature flag check for a new login flow
    if exp.BoolValue("enable_new_login_flow") {
        // New login flow is enabled
        fmt.Println("New login flow is enabled")
        return true
    }
    // Fallback to old login flow
    fmt.Println("Using old login flow")
    return false
}

func main() {
    if someFunc() {
        fmt.Println("Proceeding with new login flow...")
        // ... code for new login flow
    } else {
        fmt.Println("Proceeding with old login flow...")
        // ... code for old login flow
    }
}
