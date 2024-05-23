enum class FeatureFlags {
    STALE_FLAG,
    FEATURE_B
    ;
}

fun isStaleFeatureFlagEnabled() =
    featureService.isEnabled(FeatureFalgs.STALE_FLAG.name)

class Sample {

    fun someMethod() {

        every {
            featureService.isEnabled(FeatureFlags.STALE_FLAG)
        } returns true

        val a = FeatureFlags.STALE_FLAG

        every {
            a.isEnabled(featureService)
        } returns true

        every {
            isStaleFeatureFlagEnabled()
        } returns true

        every {
            featureService.isEnabled(STALE_FEATURE_FLAG_NAME)
        } returns false
    }

    companion object {
        private const val STALE_FEATURE_FLAG_NAME = "STALE_FLAG"
    }
}
