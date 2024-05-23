enum class FeatureFlags {
    STALE_FLAG,
    FEATURE_B
    ;
}

fun isFeatureEnabled_simple_enum() =
    featureService.isEnabled(STALE_FEATURE_NAME.name)

fun isNotFeatureEnabled_simple_enum() =
    featureService.isEnabled(FeatureFlags.STALE_FEATURE_NAME)

fun main() {
    if (isFeatureEnabled_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }
}
