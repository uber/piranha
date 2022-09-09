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

// swiftlint:disable custom_rules

import Foundation

enum ExperimentNamesSwift: String, ExperimentKeying {
    //comment0
    case random_flag //comment1

    //comment4
    case test_experiment_suffix //comment5
    // comment 11
    case test_second_experiment 
    
    case random_flag //comment6
    case test_experiment1 // comment7
    
    case random_flag //comment8
    case test_experiment1 // comment9

    case random1
    case random2
    case random3
    case random4

    case random6

    case random7

    case random8
    case random9

    var asString: String {
        return String(describing: self)
    }
}