enum class FeatureFlags(val flagName: String) {
    FEATURE_A("STALE_FLAG"),
    FEATURE_B("OTHER_FLAG")
    ;
}

fun isStaleFeatureFlagEnabled() =
    featureService.isEnabled(FeatureFlags.FEATURE_A.name)

class Sample {

    fun someMethod() {

        every {
            featureService.isEnabled(FeatureFlags.FEATURE_A)
        } returns true

        val a = FeatureFlags.FEATURE_A

        log.genInfo(
            PROCESS to FeatureFlags.FEATURE_A,
            VARIABLE to "hello world",
        )

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
