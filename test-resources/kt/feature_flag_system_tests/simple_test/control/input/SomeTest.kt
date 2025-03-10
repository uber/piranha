class Test {
    fun isNotEnabled() =
        featureService.isNotEnabled(STALE_FLAG)

    fun isMetaEnabled() = isNotEnabled()

    fun test() {
        if (isNotEnabled()) {
            println("hello world")
        }
        if (isMetaEnabled()) {
            println("hello world")
        }
    }

    enum class Features {
        STALE_FLAG,
        ANOTHER_FLAG
    }
}
