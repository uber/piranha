class SampleClassWithProperty {

    fun isFeatureEnabled() =
        featureService.isEnabled(FEATURE)

    fun main() {
        if (isFeatureEnabled()) {
            print("hello")
            return
        }
        print("Hello world!")
    }

    companion object {
        const val FEATURE = Feature(name = "STALE_FLAG")
        const val ANOTHER_FEATURE = Feature(name = "ANOTHER_STALE_FLAG")
    }
}
