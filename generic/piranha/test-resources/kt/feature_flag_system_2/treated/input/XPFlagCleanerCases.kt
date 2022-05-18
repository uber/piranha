package com.uber.input

import com.uber.BoolParameter
import java.lang.annotation.Retention
import java.lang.annotation.RetentionPolicy

internal class XPFlagCleanerPositiveCases {
    enum class TestExperimentName {
        // BUG: Diagnostic contains: Cleans stale XP flags
        STALE_FLAG
    }

    enum class AnotherTestExperimentName {
        // BUG: Diagnostic contains: Cleans stale XP flags
        @Autorollout
        STALE_FLAG
    }

    @Retention(RetentionPolicy.RUNTIME)
    annotation class Autorollout(val staged: Boolean = false)

    @Retention(RetentionPolicy.RUNTIME)
    @Target(AnnotationTarget.FUNCTION, AnnotationTarget.PROPERTY_GETTER, AnnotationTarget.PROPERTY_SETTER)
    internal annotation class ToggleTesting(val treated: Array<TestExperimentName>)

    private val experimentation: XPTest = XPTest()
    private var tBool = false
    fun conditional_contains_stale_flag(): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        }

        if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            // comment0
            return 1000
        } else {
            // comment1
            return 2000
        }
    }

    fun conditional_with_else_contains_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun complex_conditional_contains_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        if (true || tBool && experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun other_api_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        if (experimentation.isFlagTreated(TestExperimentName.STALE_FLAG)) {
            println("Hello World")
        } else {
            println("Hi world")
        }
    }

    fun assignments_containing_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)

        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && true

        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || true

        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) || tBool

        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true)
    }

    fun return_contains_stale_flag(): Boolean {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)
    }

    fun condexp_contains_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        tBool = if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) true else false
    }

    fun misc_xp_apis_containing_stale_flag() {
        // BUG: Diagnostic contains: Cleans stale XP flags
        if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {
        }

        // BUG: Diagnostic contains: Cleans stale XP flags
        experimentation.putToggleEnabled(TestExperimentName.STALE_FLAG)

        // BUG: Diagnostic contains: Cleans stale XP flags
        experimentation.includeEvent(TestExperimentName.STALE_FLAG)

        // BUG: Diagnostic contains: Cleans stale XP flags
        experimentation.putToggleDisabled(TestExperimentName.STALE_FLAG)

        // BUG: Diagnostic contains: Cleans stale XP flags
        if (experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG) && (tBool || true)) {
        }
    }

    fun return_within_if_basic(): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
            20
        } else 30
    }

    fun return_within_if_additional(x: Int): Int {
        if (x == 0) {
            // BUG: Diagnostic contains: Cleans stale XP flags
            return if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                0
            } else 75
        }
        if (x == 1) // BUG: Diagnostic contains: Cleans stale XP flags
            return if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                1
            } else {
                76
            }
        if (x == 2) {
            var y = 3
            // BUG: Diagnostic contains: Cleans stale XP flags
            if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                y++
                return y
            }
            return y + 10
        }
        if (x == 3) {
            var z = 4
            // BUG: Diagnostic contains: Cleans stale XP flags
            if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                z++
            } else {
                z = z * 5
                return z + 10
            }
            return z
        }
        return 100
    }

    @ToggleTesting(treated = [TestExperimentName.STALE_FLAG]) // BUG: Diagnostic contains: Cleans stale XP flags
    fun annotation_test() {
    }

    var injectedExperimentsShouldBeDeleted: XPTest? = null
    private fun testRemovingInjectField(): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (injectedExperimentsShouldBeDeleted!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    var injectedExperimentsMultipleUses: XPTest? = null
    private fun randomSet(x: XPTest) {
        injectedExperimentsMultipleUses = x
    }

    private fun testNotRemovingInjectField(): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (injectedExperimentsMultipleUses!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    fun unusedParamTestWithDeletion(x: XPTest): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (x.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    fun unusedParamTestWithoutDeletion(x: XPTest?): Int {
        if (x != null) {
            // just another use to prevent deletion of this parameter.
        }

        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (x!!.isToggleEnabled(TestExperimentName.STALE_FLAG)) 1 else 2
    }

    private fun testMultipleCalls(x: Int) {
        if (x > 0) {
            // BUG: Diagnostic contains: Cleans stale XP flags
            experimentation.includeEvent(TestExperimentName.STALE_FLAG)
            // BUG: Diagnostic contains: Cleans stale XP flags
            if (experimentation.isToggleEnabled(TestExperimentName.STALE_FLAG)) {
                // comment0
                return
            } else {
                // comment1
                return
            }
        }

        // do something here
        return
    }

    fun or_compounded_with_not(x: Int, extra_toggle: Boolean): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (extra_toggle || !experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
            0
        } else {
            1
        }
    }

    fun remove_else_if(extra_toggle: Boolean): Int {
        // BUG: Diagnostic contains: Cleans stale XP flags
        return if (extra_toggle) {
            0
        } else if (experimentation.isToggleDisabled(TestExperimentName.STALE_FLAG)) {
            1
        } else {
            2
        }
    }

    internal inner class XPTest {
        fun isToggleEnabled(x: TestExperimentName?): Boolean {
            return true
        }

        fun putToggleEnabled(x: TestExperimentName?): Boolean {
            return true
        }

        fun includeEvent(x: TestExperimentName?): Boolean {
            return true
        }

        fun isToggleDisabled(x: TestExperimentName?): Boolean {
            return true
        }

        fun putToggleDisabled(x: TestExperimentName?): Boolean {
            return true
        }

        fun isFlagTreated(x: TestExperimentName?): Boolean {
            return true
        }

        fun isToggleInGroup(x: TestExperimentName?): Boolean {
            return true
        }

    }
}