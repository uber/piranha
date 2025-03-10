enum class FeatureFlags(val featureName: String) {
    FEATURE_B("FEATURE_B")
    ;

    companion object {
        val FeatureB = Feature(FEATURE_B.key)
    }
}

fun Map<String, String>.isSomeExpressionEnabled(key: SomeType, defaultValue: Boolean): Boolean {
    return false
}

fun main() {
    doSomething()
    doSomething()

    println("Hello world!")
    println("Hi world!")
    println("Hello world!")
}
