class Sample {

    override fun isCompositeEnabled(userId: UserId, country: Country, param: Param): Boolean =
        isFeatureEnabled(SOME_OTHER_FEATURE.withUserId(userId.value)) ||
            COUNTRY_LIST.contains(country) ||
            (
                country.isFeatureEnabled(param)
            )

    fun someMethod() {
        println("Hello world!")
        println("Hi world!")
        println("Hi world!")
        println("Hi world!")
    }

    companion object {
    }
}
