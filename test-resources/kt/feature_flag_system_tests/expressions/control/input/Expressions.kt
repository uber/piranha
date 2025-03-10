data class Test(val a: Int) { }
data class Test2(val b: Int)

class Sample {

    fun isFeatureEnabled_ext_based(userId: UserId) =
        STALE_FLAG_FEATURE.isEnabled(featureService, UserId(userId))

    fun isNotFeatureEnabled_ext_based_literal() =
        "STALE_FLAG".isNotEnabled(featureService)

    fun isNotFeatureEnabled_literal() =
        featureService.isNotEnabled("STALE_FLAG")

    fun isSomeFeatureEnabledForUser() =
        featureService.isFeatureEnabledForUser("STALE_FLAG")

    fun featureIsEnabled() =
        featureService.isNotEnabled("STALE_FLAG")

    fun featureIsEnabledMixed() =
        featureService.isEnabled("STALE_FLAG") && featureService.isEnabled("FEATURE_B")

    fun featureIsEnabledNegativeMixed(someCondition: Boolean) =
        (someCondition || anotherCondition) &&
        !featureService.isEnabled("STALE_FLAG") &&
        condition != true

    fun featureIsEnabledNegativeMixed2(someCondition: Boolean) =
        (someCondition && anotherCondition) ||
        !featureService.isNotEnabled("STALE_FLAG") ||
        condition != true

    fun featureIsEnabledUsingFeatureBuilders() =
        featureService.isEnabled(Feature("STALE_FLAG").withUserId(userId.value), true)

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
            isFeatureEnabled_ext_based()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if ("STALE_FLAG".isNotEnabled(featureService)) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (featureIsEnabled()) {
            println("Hi world!")
        }

        featureService.execute(STALE_FLAG_FEATURE) {
            println("hello world")
        }

        featureService.runIfEnabled(STALE_FLAG_FEATURE) {
            val i = 0
            doSomething()
        }.fold(
            { println("hello world") },
            { println("hi world") }
        )

        featureService.runIfNotEnabled(STALE_FLAG_FEATURE) {
            doSomething()
        }.fold(
            { println("hello world") },
            { println("hi world") }
        )

        if (featureIsEnabledMixed()) {
            println("hello world")
        }

        if (featureIsEnabledUsingFeatureBuilders()) {
            println("hello world")
        }

        if (isSomeFeatureEnabledForUser(userId)) {
            println("ok")
        }

        log.info(
            "test" to STALE_FLAG_FEATURE,
            "hello" to "world"
        )
    }

    fun Map<String, String>.isSomeExpressionEnabled(key: String, defaultValue: Boolean): Boolean {
        return key == STALE_FLAG_FEATURE && defaultValue
    }

    companion object {
        private const val STALE_FLAG_FEATURE = "STALE_FLAG"
    }
}
