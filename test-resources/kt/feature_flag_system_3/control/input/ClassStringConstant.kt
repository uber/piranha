class Sample {

    fun isFeatureEnabled() =
        featureService.isEnabled(STALE_FLAG_FEATURE)

    fun isNotFeatureEnabled_2() =
        featureService.isNotEnabled(STALE_FLAG_FEATURE)

    fun someMethod() {
        if (isFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled_2()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (service.isNotFeatureEnabled_2()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }
    }

    companion object {
        private const val STALE_FLAG_FEATURE = "STALE_FLAG"
    }
}
