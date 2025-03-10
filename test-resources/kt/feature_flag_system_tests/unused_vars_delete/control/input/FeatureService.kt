class FeatureService {
    fun isStaleFeatureEnabled() = featureService.isEnabled(STALE_FLAG)
    fun isNotStaleFeatureEnabled() = featureService.isEnabled(FEATURE_B)
}
