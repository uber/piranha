enum class FeatureFlags(val someTestValue: Boolean) {
    STALE_FLAG(true),
    FEATURE_B(true)
    ;
}

enum class FeatureFlagsAnother(val someTestValue: String) {
    ANOTHER_REPRESENTATION_OF_STALE_FLAG("STALE_FLAG"),
    ;
}
