enum class FeatureFlags {
    STALE_FLAG,
    FEATURE_B
    ;
}

class Sample {

    fun someMethod() {
        val a = "STALE_FLAG"
        if (featureService.isEnabled(a)) {
            println("Hello world!")
        }

        val b = FeatureFlags.STALE_FLAG
        if (featureService.isEnabled(b)) {
            println("Hi world!")
        }

        val c = featureService.isEnabled(a)
        if (c) {
            println("Hi world!")
        }
    }
}
