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
package com.uber.input

import com.uber.Parameter
import com.uber.SomeOtherInterface
import com.uber.SomeParamRev
import com.uber.StaticMthds

internal class XPMethodChainCases {
    // should not match instance method where nested invocation is not a member select tree.
    fun testDontMatchNonInstanceNested() {
        // Does not Match
        if (StaticMthds.isStaleFeature().cachedValue) {
            print("!!")
        }
    }

    companion object {
        fun foobar(cp: Parameter) {
            val sp = SomeParameter.create()
            // Matches API
            println("!")

            // Does not match API
            if (sp.isOtherFlag().cachedValue) {
                println("!!!")
            }
            if (sp.isOtherFlag().cachedValue) {
                println("!!!")
            }
            
            println("simplify if and keep statement")
            
            // simple_identifier || true
            println("Test for identifier || true!!!")
            
            val spr = SomeParamRev.create(cp)
            // Does not match API- is reverse order
            if (spr.cachedValue.isStaleFeature()) {
                println("!!!!")
            }
            // Does not match API- matches partially
            if (spr.cachedValue != null) {
                println("!!!!!")
            }
            val sot = SomeOtherInterface.create(cp)
            // Does not match API- matches partially
            if (sot.isStaleFeature() != null) {
                println("!!")
            }
            // Does not Match - static method invocation
            if (StaticMthds.isStaleFeature().cachedValue) {
                print("!!")
            }
            println("done!")

            // Do not match API
            cp.put(sp.isOtherFlag(), true)
            cp.put(sp.isOtherFlag(), false)
        }
    }
}
