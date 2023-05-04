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

import java.lang.annotation.Retention
import java.lang.annotation.RetentionPolicy

internal class XPFlagCleanerPositiveCases {
    enum class TestExperimentName {
        SOME_FLAG
    }

    enum class AnotherTestExperimentName {
        SOME_OTHER_FLAG
    }

    enum class TestEmptyEnum {
    }

    @Retention(RetentionPolicy.RUNTIME)
    annotation class Autorollout(val staged: Boolean = false)

    @Retention(RetentionPolicy.RUNTIME)
    @Target(AnnotationTarget.FUNCTION, AnnotationTarget.PROPERTY_GETTER, AnnotationTarget.PROPERTY_SETTER)
    internal annotation class ToggleTesting(val treated: Array<TestExperimentName>)

    private val experimentation: XPTest? = null
    private var tBool = false
    fun conditional_contains_stale_flag() {
    }

    fun conditional_with_else_contains_stale_flag() {
        println("Hi world")

        println("Hi world 1")

        println("Hello World 2")

        println("Hi world 3")
    }

    fun complex_conditional_contains_stale_flag() {
        println("Hello World")

    }

    fun other_api_stale_flag() {
        println("Hi world")
    }

    fun assignments_containing_stale_flag() {
        tBool = false
        tBool = false

        tBool = true

        tBool = tBool
        tBool = false
    }

    fun return_contains_stale_flag(): Boolean {
        return false
    }

    fun condexp_contains_stale_flag() {
        tBool = false
    }

    fun misc_xp_apis_containing_stale_flag() {
        if ((tBool || true)) {
            println("Hello World")
        }
    }

    fun return_within_if_basic(): Int {
        return 30
    }

    fun return_within_if_additional(x: Int): Int {
        if (x == 0) {
            return 75
        }
        if (x == 1)
            return 76
        if (x == 2) {
            var y = 3
            return y + 10
        }
        if (x == 3) {
            var z = 4
            z = z * 5
            return z + 10
        }
        return 100
    }

    private fun testRemovingInjectField(): Int {
        return 2
    }

    var injectedExperimentsMultipleUses: XPTest? = null
    private fun randomSet(x: XPTest) {
        injectedExperimentsMultipleUses = x
    }

    private fun testNotRemovingInjectField(): Int {
        return 2
    }

    fun unusedParamTestWithDeletion(): Int {
        return 2
    }

    fun unusedParamTestWithoutDeletion(x: XPTest?): Int {
        if (x != null) {
        }
        return 2
    }

    private fun testMultipleCalls(x: Int): Int {
        if (x > 0) {
            return 2000
        }

        return 3000
    }

    fun or_compounded_with_not(x: Int, extra_toggle: Boolean): Int {
        if (extra_toggle) {
            return 0
        } else {
            return 1
        }
    }

    fun remove_else_if(extra_toggle: Boolean): Int {
        return if (extra_toggle) {
            0
        } else {
            1
        }
    }
}
