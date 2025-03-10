import packagea.packageb.STALE_FLAG
import packagea.STALE_FLAG

import test.packagea.FeatureToggleInner.ANOTHER_STALE_FLAG
import FeatureToggleInner.ANOTHER_STALE_FLAG
import packagea.FeatureToggleInner.ANOTHER_STALE_FLAG

sealed class FeatureToggle

object FEATURE_A : FeatureToggle("feature_a")
object STALE_FLAG : FeatureToggle("STALE_FLAG")

sealed class FeatureToggleInner {
    object FEATURE_A : FeatureToggleInner("feature_a")
    object ANOTHER_STALE_FLAG : FeatureToggleInner("STALE_FLAG")
}


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
