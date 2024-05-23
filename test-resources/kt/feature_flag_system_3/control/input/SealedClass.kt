sealed class FeatureToggle

object FEATURE_A : FeatureToggle
object STALE_FLAG : FeatureToggle

fun isFeatureEnabled_simple_sealed_class() =
    featureService.isEnabled(STALE_FLAG)

fun isNotFeatureEnabled_simple_sealed_class() =
    featureService.isEnabled(STALE_FLAG)

fun main() {
    if (isFeatureEnabled_simple_sealed_class()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_simple_sealed_class()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }
}
