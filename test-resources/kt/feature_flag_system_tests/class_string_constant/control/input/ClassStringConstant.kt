class Sample {

    fun isFeatureEnabled_1() =
        featureService.isEnabled(STALE_FLAG_FEATURE)

    fun isNotFeatureEnabled_2() =
        featureService.isNotEnabled(STALE_FLAG_FEATURE)

    fun isNotFeatureEnabled_3() =
        isNotEnabled(STALE_FLAG_FEATURE)

    override fun isCompositeEnabled(userId: UserId, country: Country, param: Param): Boolean =
        isFeatureEnabled(SOME_OTHER_FEATURE.withUserId(userId.value)) ||
        COUNTRY_LIST.contains(country) ||
        (
            country.isFeatureEnabled(param) && isFeatureEnabled(STALE_FLAG_FEATURE.withUserId(userId.value))
        )

    fun someMethod() {
        if (isFeatureEnabled_1()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled_2()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled_3()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (service.isNotFeatureEnabled_2()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        fermiumStub.setNotFeatureResponse(Feature(STALE_FLAG_FEATURE), true)
    }

    companion object {
        const val STALE_FLAG_FEATURE = "STALE_FLAG"
    }
}
