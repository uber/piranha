class Sample {

    fun isFeatureEnabled_ext_based(userId: UserId) =
        STALE_FLAG_FEATURE.isEnabled(featureService, UserId(userId))

    fun isNotFeatureEnabled_ext_based_literal() =
        "STALE_FLAG".isNotEnabled(featureService)

    fun isNotFeatureEnabled_literal() =
        featureService.isNotEnabled("STALE_FLAG")

    fun someMethod() {
        if (isFeatureEnabled_ext_based(userId)) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled_ext_based_literal()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled_literal()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (featureService.isNotEnabled("STALE_FLAG")) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (STALE_FLAG_FEATURE.isEnabled(featureService, UserId(userId)) &&
            isFeatureEnabled_ext_based()
        ) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if ("STALE_FLAG".isNotEnabled(featureService)) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }
    }

    companion object {
        private const val STALE_FLAG_FEATURE = "STALE_FLAG"
    }
}
