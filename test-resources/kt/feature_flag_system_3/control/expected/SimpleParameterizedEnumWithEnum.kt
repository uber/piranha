enum class FeatureFlags {
    ANOTHER_FLAG
    ;
}

enum class SecondaryEnum(val featureName: String) {
    FEATURE_B(ANOTHER_FLAG)
    ;
}

fun main() {
    println("Hello world!")
    println("Hi world!")
    println("Hello world!")
}
