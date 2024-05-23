enum class FeatureFlags(val featureName: String) {
    STALE_FEATURE_NAME("STALE_FLAG"),
    FEATURE_B("FEATURE_B")
    ;
}

fun isFeatureEnabled() =
    featureService.isEnabled(STALE_FEATURE_NAME)

fun isNotFeatureEnabled() =
    featureService.isEnabled(STALE_FEATURE_NAME)

fun main() {
    if (isFeatureEnabled()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (featureService.isEnabled(STALE_FEATURE_NAME)) {
        println("Hello world!")
    }
    if (featureService.isNotEnabled(STALE_FEATURE_NAME)) {
        println("Hi world!")
    }
}
