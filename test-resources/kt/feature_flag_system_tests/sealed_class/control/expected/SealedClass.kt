sealed class FeatureToggle

object FEATURE_A : FeatureToggle("feature_a")

sealed class FeatureToggleInner {
    object FEATURE_A : FeatureToggleInner("feature_a")
}

fun main() {
    println("Hello world!")
    println("Hi world!")
}
