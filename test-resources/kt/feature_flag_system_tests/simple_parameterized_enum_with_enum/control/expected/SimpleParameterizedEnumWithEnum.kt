const val TEST = "AccountV3EventRecoverer"

class Test {

    fun main() {
        println("Hello world!")
        println("Hi world!")
        println("Hello world!")
        println("Hi world!")
    }

    enum class SecondaryEnum(val somePrefix: String, val featureName: String) {
        FEATURE_B(TEST, ANOTHER_FLAG.value)
        ;
    }
}
