enum class FeatureFlags(val someTestValue: Boolean) {
    STALE_FLAG(true),
    FEATURE_B(true)
    ;
}

fun test() {
    if (featureService.isEnabled(STALE_FLAG)) {
        print("hello world")
    }
}
