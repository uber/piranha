package com.uber.input

import java.lang.annotation.Retention
import java.lang.annotation.RetentionPolicy

internal class XPFlagCleanerPositiveCases {
     enum class TestExperimentName { 
    }

    enum class AnotherTestExperimentName { 
    }

    @Retention(RetentionPolicy.RUNTIME)
    annotation class Autorollout(val staged: Boolean = false)

    @Retention(RetentionPolicy.RUNTIME)
    @Target(AnnotationTarget.FUNCTION, AnnotationTarget.PROPERTY_GETTER, AnnotationTarget.PROPERTY_SETTER)
    internal annotation class ToggleTesting(val treated: Array<TestExperimentName>)

    private val experimentation: XPTest? = null
    private var tBool = false
    fun conditional_contains_stale_flag() {
        println("Hello World")
    }

    fun conditional_with_else_contains_stale_flag() {
        println("Hello World")
    }

     fun complex_conditional_contains_stale_flag() {
        println("Hello World")
    }

    fun other_api_stale_flag() {
        println("Hello World")
    }

    fun assignments_containing_stale_flag() {
        tBool = true
        tBool = true
        tBool = true
        tBool = true
        tBool = (tBool || true)
    }

    fun return_contains_stale_flag(): Boolean {
      // FIXME:
       return true
    }

    fun condexp_contains_stale_flag() {
        tBool = true
    }

    fun misc_xp_apis_containing_stale_flag() {
           if ((tBool || true)) {
        }
    }

    fun return_within_if_basic(): Int {
        return 20
    }

    fun return_within_if_additional(x: Int): Int {
        if (x == 0) {
            return 0
        }
        if (x == 1) return 1
        if (x == 2) {
            var y = 3
            y++
            return 1000
        }
        if (x == 3) {
            var z = 4
            z++
            return 10000
        }
        return 100
    }

    

    private fun testRemovingInjectField(): Int {
         return 1
    }

    var injectedExperimentsMultipleUses: XPTest? = null
    private fun randomSet(x: XPTest) {
        injectedExperimentsMultipleUses = x
    }

    private fun testNotRemovingInjectField(): Int {
        
        return 1
    }

    
    fun unusedParamTestWithDeletion(): Int {
        
        return 1
    }

    fun unusedParamTestWithoutDeletion(x: XPTest?): Int {
        if (x != null) {
        }

        
        return 1
    }


    private fun testMultipleCalls(x: Int):Int {
        if (x > 0) {
            return 1000
        }

        // do something here
        return 3000
    }

    fun or_compounded_with_not(x: Int, extra_toggle: Boolean): Int {
        return 0
    }

    fun remove_else_if(extra_toggle: Boolean): Int {
        return if (extra_toggle) {
            0
        } else {
            2
        }
    }
}