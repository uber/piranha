enum class FeatureFlags(val flagName: String) {
    FEATURE_B("OTHER_FLAG")
    ;
}

class Sample {

    fun someMethod() {
        log.genInfo(
            VARIABLE to "hello world",
        )
    }

    companion object {
    }
}
