/*
Copyright (c) 2022 Uber Technologies, Inc.

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
        SOME_FLAG,
        STALE_FLAG
    }

    enum class AnotherTestExperimentName {
        @Autorollout
        STALE_FLAG,
        SOME_OTHER_FLAG
    }

    enum class YetAnotherTestExperimentName {
        @Autorollout
        STALE_FLAG,
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
        if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        }
    }

    fun conditional_with_else_contains_stale_flag() {
        if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun complex_conditional_contains_stale_flag() {
        if (true || tBool && experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun other_api_stale_flag() {
        if (experimentation!!.isFlagTreated(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun assignments_containing_stale_flag() {
        tBool = experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)


        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && true


        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || true


        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || tBool


        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true)
    }

    fun return_contains_stale_flag(): Boolean {
        return experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)
    }

    fun condexp_contains_stale_flag() {
        tBool = if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) true else false
    }

    fun misc_xp_apis_containing_stale_flag() {
        if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {
        }


        experimentation.putToggleEnabled(TestExperimentName.STALE_FLAG)


        experimentation.includeEvent(TestExperimentName.STALE_FLAG)


        experimentation.putToggleDisabled(TestExperimentName.STALE_FLAG)


        if (experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {
            println("Hello World")
        }
    }

    fun return_within_if_basic(): Int {
        return if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            20
        } else 30
    }

    fun return_within_if_additional(x: Int): Int {
        if (x == 0) {
            return if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                0
            } else 75
        }
        if (x == 1)
            return if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                1
            } else {
                76
            }
        if (x == 2) {
            var y = 3

            if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                y++
                return 1000
            }
            return y + 10
        }
        if (x == 3) {
            var z = 4

            if (experimentation!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                z++
            } else {
                z = z * 5
                return z + 10
            }
            return 10000
        }
        return 100
    }

    private fun testRemovingInjectField(): Int {
        return if (injectedExperimentsShouldBeDeleted!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    var injectedExperimentsMultipleUses: XPTest? = null
    private fun randomSet(x: XPTest) {
        injectedExperimentsMultipleUses = x
    }

    private fun testNotRemovingInjectField(): Int {
        return if (injectedExperimentsMultipleUses!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    fun unusedParamTestWithDeletion(): Int {
        return if (x.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    fun unusedParamTestWithoutDeletion(x: XPTest?): Int {
        if (x != null) {
        }


        return if (x!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    private fun testMultipleCalls(x: Int): Int {
        if (x > 0) {
            experimentation!!.includeEvent(TestExperimentName.STALE_FLAG)

            if (experimentation1.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                return 1000
            } else {
                return 2000
            }
        }


        return 3000
    }

    fun or_compounded_with_not(x: Int, extra_toggle: Boolean): Int {
        if (extra_toggle || !experimentation!!.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
            return 0
        } else {
            return 1
        }
    }

    fun remove_else_if(extra_toggle: Boolean): Int {
        return if (extra_toggle) {
            0
        } else if (experimentation!!.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
            1
        } else {
            2
        }
    }
}