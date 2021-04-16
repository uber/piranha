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

import Foundation

struct PiranhaConfig: Codable {
    enum CodingKeys: String, CodingKey {
        case methods = "methodProperties"
    }
    
    let methods: [Method]
}

struct Method: Codable, Equatable {
    let methodName: String
    let flagType: FlagType
    let flagIndex: Int
    let groupIndex: Int?
    
    enum FlagType: String, Codable {
        case treated
        case control
        case testing
    }
}
