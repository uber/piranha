enum class Features {
    FEATURE_A("STALE_FLAG")
}

class SampleTests {
    private val featureA = Features.FEATURE_A

    @BeforeAll
    fun simpleLambda() = featureService.isEnabled(FEATURE_A)

    fun tearDown() = featureStub.setEnabled(featureA)
}


class Sample {
    fun isFeatureEnabled() = featureService.isEnabled(FEATURE_A)

    fun test() {
        featureService.check('test', Sample::isFeatureEnabled)
    }
}
