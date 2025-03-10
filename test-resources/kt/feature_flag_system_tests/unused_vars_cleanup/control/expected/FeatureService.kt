class FeatureService {
    fun isNotStaleFeatureEnabled() = featureService.isEnabled(FEATURE_B)
}
