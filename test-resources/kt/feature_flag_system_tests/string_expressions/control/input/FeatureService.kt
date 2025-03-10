fun test() {
    val result = featureService.isEnabled(STALE_FLAG)
    val isFeatureEnabled = isStaleFeatureEnabled()
    val someCompositeFeatureEnabled = someTrueVar && isFeatureEnabled
    val someStupidVar = isFeatureEnabled
    val someVar = STALE_FLAG
    logger.info("isFeatureEnabled=$STALE_FLAG isFeatureEnabled=$isFeatureEnabled someVar=${someVar}")
    if (someStupidVar) {
        print("ok")
    }
}

enum class FeatureFlags {
    STALE_FLAG,
    ANOTHER_FLAG
    ;
}

fun isStaleFeatureEnabled() = featureService.isEnabled(STALE_FLAG)
