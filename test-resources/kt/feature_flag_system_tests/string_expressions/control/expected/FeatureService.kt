fun test() {
    val someCompositeFeatureEnabled = someTrueVar
    logger.info("isFeatureEnabled=$ isFeatureEnabled=$true someVar=${}")
    print("ok")
}

enum class FeatureFlags {
    ANOTHER_FLAG
    ;
}
