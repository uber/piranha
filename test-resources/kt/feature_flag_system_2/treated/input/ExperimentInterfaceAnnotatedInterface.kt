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

import com.uber.BoolParam
import com.uber.BoolParameter
import com.uber.Parameter
import com.uber.ParameterDefinition

@ParameterDefinition(namespace = "some_long_name")
internal interface SomeOtherParameter {
    @BoolParam(key = "STALE_FLAG")
    fun isStaleFeature(): BoolParameter

    @BoolParam(key = "other_flag", namespace = "some_long_name")
    fun isOtherFlag(): BoolParameter

    companion object {
        fun create(): SomeOtherParameter {
            return object : SomeOtherParameter {
                override fun isStaleFeature(): BoolParameter {
                    TODO("Not yet implemented")
                }

                override fun isOtherFlag(): BoolParameter {
                    TODO("Not yet implemented")
                }
            }
        }
    }

}
