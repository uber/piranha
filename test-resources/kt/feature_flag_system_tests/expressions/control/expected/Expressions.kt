data class Test(val a: Int) { }
data class Test2(val b: Int)

class Sample {


    fun featureIsEnabledMixed() =
        featureService.isEnabled("FEATURE_B")

    fun featureIsEnabledNegativeMixed(someCondition: Boolean) =
        false

    fun featureIsEnabledNegativeMixed2(someCondition: Boolean) =
        true

    fun someMethod() {
        println("Hello world!")
        println("Hi world!")
        println("Hi world!")
        println("Hi world!")
        println("Hello world!")
        println("Hi world!")
        println("Hi world!")
        println("hello world")

        val i = 0
        doSomething().fold(
            { println("hello world") },
            { println("hi world") }
        )

        if (featureIsEnabledMixed()) {
            println("hello world")
        }

        println("hello world")
        println("ok")

        log.info(
            "hello" to "world"
        )
    }

    fun Map<String, String>.isSomeExpressionEnabled(key: String, defaultValue: Boolean): Boolean {
        return false
    }

    companion object {
    }
}
