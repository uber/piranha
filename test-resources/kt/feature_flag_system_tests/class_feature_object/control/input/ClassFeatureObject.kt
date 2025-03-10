class Sample {

    override fun isStaleFeatureEnabled() {
        return featureService.isEnabled(STALE_FLAG_FEATURE, false)
    }

    fun someMethod() {
        if (isStaleFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }
    }

    companion object {
        private val STALE_FLAG_FEATURE = Feature(name = "STALE_FLAG")
    }
}
