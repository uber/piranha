class Test {
    fun isEnabled() =
        featureService.isEnabled(FEATURE_A) && featureService.isEnabled(STALE_FLAG)


    fun test() {
        if (isEnabled()) {
            println("hello world")
        }
    }

    enum class Features {
        STALE_FLAG,
        ANOTHER_FLAG
    }
}
