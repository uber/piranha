/**
 *    Copyright (c) 2021 Uber Technologies, Inc.
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

import SwiftSyntax

extension ConditionElementListSyntax {
    /// Returns a new Element with trailing comma removed.
    /// For ex: "x," becomes  "x"
    mutating func removeLastTrailingCommaIfNeeded() {
        guard let lastElement = last,
              let trailingComma = lastElement.trailingComma else { return }
        let element = ConditionElementSyntax { (builder) in
            builder.useCondition(lastElement.condition.withTrailingTrivia(trailingComma.trailingTrivia))
        }
        self = self.replacing(childAt: lastElement.indexInParent,
                              with: element)
    }
}
