/**
 * Copyright (c) 2022 Uber Technologies, Inc.
 *
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of the License at
 *
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the
 * License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
 * express or implied. See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.uber.input

internal class XPFlagCleanerPositiveCases {
    private val experimentation: SomeParameter = SomeParameter.create()
    
    private var ftBool: Boolean = experimentation.isOtherFlag().cachedValue
    fun conditional_contains_stale_flag() {
        if (experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        }
    }

    fun conditional_with_else_contains_stale_flag() {
        if (experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_tbool() {
        val tBool: Boolean = experimentation.isOtherFlag().cachedValue
        if (tBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_tbool(a: Int) {
        val tBool: Boolean = experimentation.isOtherFlag().cachedValue
        if (tBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_tbool(a: Int, b: Int) {
        val tBool: Boolean = experimentation.isOtherFlag().cachedValue
        if (!tBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_tbool_reassigned(a: Int, z: Int) {
        // Currently if there is another assignment, variable will not be inlined.
        var tBool: Boolean = experimentation.isOtherFlag().cachedValue
        tBool = abc() && tBool
        if (!tBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun abc(): Boolean = true

    fun conditional_with_else_contains_stale_flag_tbool_reassigned_to_same_val(a: Int, z: Int) {
        var tBool: Boolean = experimentation.isOtherFlag().cachedValue
        tBool = true
        if (!tBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_ftbool(a: Int) {
        if (ftBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun conditional_with_else_contains_stale_flag_tbool_reassigned_ftbool(a: Int, z: Int) {
        // Currently if there is another assignment, variable will not be inlined.
        ftBool = experimentation.isOtherFlag().cachedValue
        if (!ftBool && experimentation.isOtherFlag().cachedValue) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }
}