class AnotherClass(
    private val featureService: FeatureService,
) {

    private val someProp: String

    fun main2() {
        if (featureService.isNotStaleFeatureEnabled()) {
            print("hello")
        }
    }
}
