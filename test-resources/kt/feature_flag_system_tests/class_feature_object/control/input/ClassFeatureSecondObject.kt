class Sample : SampleInterface {

    fun isStaleFeatureEnabled() {
        return featureService.isEnabled(STALE_FLAG_FEATURE, false)
    }

    fun someMethod() {
        if (isStaleFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }
        featureStub.setFeature(STALE_FLAG_FEATURE, true)
    }

    companion object {
        private val STALE_FLAG_FEATURE = Feature(name = "ANOTHER_FEATURE")
    }
}
