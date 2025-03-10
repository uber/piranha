import ObjectFeatureFlags.OBJECT_FLAG_A

const val TEST = "AccountV3EventRecoverer"

fun isFeatureEnabled() =
    featureService.isEnabled(STALE_FEATURE_NAME)

fun isNotFeatureEnabled() =
    featureService.isEnabled(STALE_FEATURE_NAME)

class Test {

    fun main() {
        if (isFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (isNotFeatureEnabled()) {
            println("Hello world!")
        } else {
            println("Hi world!")
        }

        if (featureService.isEnabled(STALE_FEATURE_NAME)) {
            println("Hello world!")
        }
        if (featureService.isNotEnabled(STALE_FEATURE_NAME)) {
            println("Hi world!")
        }
        if (featureService.isEnabled(OBJECT_FLAG_A)) {
            println("Hi world!")
        }
    }

    enum class SecondaryEnum(val somePrefix: String, val featureName: String) {
        STALE_FEATURE_NAME(TEST, FLAG_A.value),
        FEATURE_B(TEST, ANOTHER_FLAG.value)
        ;
    }
}
