import FeatureFlags.STALE_FLAG

class Sample(private val featureService: FeatureService) {

    private val staleFeature = Feature(STALE_FLAG.value)

    fun someMethod() {
        if (featureService.isEnabled(staleFeature, true)) {
            println("Hello world!")
        }

        val b = staleFeature
        if (featureService.isEnabled(b)) {
            println("Hi world!")
        }
    }
}
